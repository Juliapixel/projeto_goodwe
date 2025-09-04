use std::{
    collections::HashMap,
    fmt::Display,
    io,
    net::{Ipv4Addr, SocketAddr},
    num::Wrapping,
    ops::ControlFlow,
    sync::Arc,
    time::Duration,
};

use common::{DisconnectReason, MessagePayload, PlugMessage};
use env_logger::Env;
use log::{debug, info, warn};
use tokio::{
    net::UdpSocket,
    select,
    sync::mpsc::{Receiver, Sender, channel},
    time::timeout,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    Working,
    Pinging([u8; 16]),
    Unknown,
    Dead,
}

type MsgTx = Sender<PlugMessage>;
type MsgRx = Receiver<PlugMessage>;

struct BrokerConnection {
    socket: Arc<UdpSocket>,
    addr: SocketAddr,
    rx: MsgRx,
    seq: Wrapping<u32>,
    client_seq: Wrapping<u32>,
    state: ConnectionState,
}

#[derive(Debug, Clone, Copy)]
enum ConnectionError {
    Dead,
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Connection dead")
    }
}

impl std::error::Error for ConnectionError {}

impl BrokerConnection {
    pub fn new(socket: Arc<UdpSocket>, addr: SocketAddr, rx: MsgRx) -> Self {
        Self {
            socket,
            addr,
            rx,
            seq: Wrapping(rand::random()),
            client_seq: Wrapping(0),
            state: ConnectionState::Unknown,
        }
    }

    pub async fn send(&mut self, payload: MessagePayload) -> io::Result<usize> {
        self.seq += 1;
        self.socket
            .send_to(
                &postcard::to_stdvec(&PlugMessage::new(self.seq.0, payload)).unwrap(),
                self.addr,
            )
            .await
    }

    pub async fn disconnect(&mut self, reason: DisconnectReason) -> io::Result<usize> {
        self.state = ConnectionState::Dead;
        self.send(MessagePayload::Disconnect { reason }).await
    }

    pub fn can_recv(&self) -> bool {
        self.state != ConnectionState::Dead
    }

    pub async fn recv(&mut self) -> io::Result<usize> {
        if !self.can_recv() {
            return Err(io::Error::other(ConnectionError::Dead));
        }
        let next = match timeout(
            Duration::from_millis(rand::random_range(29000..=31000)),
            self.rx.recv(),
        )
        .await
        {
            Err(_) => {
                warn!("Message receiving timed out");
                self.feed_msg(None).await
            }
            Ok(None) => {
                warn!("Message pipe dead");
                self.state = ConnectionState::Dead;
                return Ok(0);
            }
            Ok(Some(msg)) => {
                info!("Received {msg:?} from {}", self.addr);
                self.feed_msg(Some(msg)).await
            }
        };

        match next {
            ControlFlow::Continue(Some(msg)) => {
                info!("Sending {msg:?} to {}", self.addr);
                self.send(msg).await
            }
            ControlFlow::Continue(None) => Ok(0),
            ControlFlow::Break(reason) => {
                warn!("Disconnecting from {} ({reason:?})", self.addr);
                self.disconnect(reason).await
            }
        }
    }

    pub async fn feed_msg(
        &mut self,
        msg: Option<PlugMessage>,
    ) -> ControlFlow<DisconnectReason, Option<MessagePayload>> {
        use ConnectionState as Cs;
        use DisconnectReason as Dr;
        use MessagePayload as Mp;

        macro_rules! dc {
            ($thing:expr) => {
                ControlFlow::Break($thing)
            };
        }

        macro_rules! ok {
            () => {
                ControlFlow::Continue(None)
            };
            ($thing:expr) => {
                ControlFlow::Continue(Some($thing))
            };
            ($thing:expr, $state:expr) => {{
                self.state = $state;
                ok!($thing)
            }};
            (, $state:expr) => {{
                self.state = $state;
                ControlFlow::Continue(None)
            }};
        }

        if let Some(msg) = &msg
            && self.state != Cs::Unknown
        {
            if msg.seq != (self.client_seq + Wrapping(1)).0 {
                return ControlFlow::Break(Dr::SequenceError);
            } else {
                self.client_seq = Wrapping(msg.seq);
            }
        }

        match (msg.map(|m| m.payload), self.state) {
            (Some(Mp::Conn), ConnectionState::Unknown) => {
                self.client_seq = Wrapping(msg.unwrap().seq);
                ok!(Mp::ConnAck, Cs::Working)
            }
            (Some(Mp::Conn), _) => dc!(Dr::Closed),
            (Some(Mp::Disconnect { reason }), _) => {
                warn!("Client requested disconnect: {reason:?}");
                dc!(Dr::Closed)
            }
            (Some(Mp::Ping { data }), _) => ok!(Mp::Pong { data }),
            (Some(Mp::Pong { data }), ConnectionState::Pinging(d)) => {
                if data == d {
                    ok!(, Cs::Working)
                } else {
                    dc!(Dr::BadHeartbeat)
                }
            }
            (Some(Mp::Pong { data: _ }), _) => dc!(Dr::ProtocolError),
            (Some(Mp::TurnOffAck), _) => {
                /* TODO: implement notification logic */
                ok!()
            }
            (Some(Mp::TurnOn), _) => {
                /* TODO: implement notification logic */
                ok!()
            }
            (Some(Mp::StatusResp { is_on: _ }), _) => {
                /* TODO: implement notification logic */
                ok!()
            }
            (Some(m), Cs::Pinging(_d)) => {
                info!("Unhandled message: {m:?}");
                ok!()
            }
            (Some(m), Cs::Working) => {
                info!("Unhandled message: {m:?}");
                ok!()
            }
            (None, Cs::Pinging(_)) => dc!(Dr::Timeout),
            (None, Cs::Unknown) => dc!(Dr::Closed),
            (None, Cs::Working) => {
                let data = rand::random();
                ok!(Mp::Ping { data }, Cs::Pinging(data))
            }
            (_, Cs::Unknown) => dc!(Dr::Closed),
            (_, Cs::Dead) => ok!(),
        }
    }
}

async fn worker_task(rx: MsgRx, socket: Arc<UdpSocket>, addr: SocketAddr) {
    let mut conn = BrokerConnection::new(socket, addr, rx);
    loop {
        match conn.recv().await {
            Ok(n) => debug!("Wrote {n} bytes to {addr}"),
            Err(e) => {
                warn!("Connection with {addr} errored: {e:?}");
                break;
            }
        };
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("INFO"));

    let socket = Arc::new(tokio::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 8080)).await?);

    let mut sessoes = HashMap::<SocketAddr, MsgTx>::new();

    info!("Broker started");

    let ctrl_c = tokio::signal::ctrl_c();

    let mut buf = vec![];

    let mut listen_loop = async || -> anyhow::Result<()> {
        loop {
            let (bytes, addr) = socket.recv_buf_from(&mut buf).await?;
            debug!("Received: {bytes} bytes from {addr}");
            let msg = postcard::from_bytes::<common::PlugMessage>(&buf[0..bytes])?;
            if msg.payload == MessagePayload::Conn {
                let (tx, rx) = channel(16);
                tx.send(msg).await.unwrap();
                sessoes.insert(addr, tx);
                tokio::spawn(worker_task(rx, socket.clone(), addr));
            } else if let Some(conn) = sessoes.get(&addr) {
                conn.send(msg).await?;
            }
            buf.clear();
        }
    };
    select! {
        _ = listen_loop() => {},
        _ = ctrl_c => {
            info!("Received shutdown signal");
            for s in sessoes.into_values() {
                drop(s)
            }
        },
    };
    Ok(())
}
