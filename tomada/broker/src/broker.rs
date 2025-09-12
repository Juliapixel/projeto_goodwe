use std::{
    collections::HashMap, fmt::Display, net::SocketAddr, num::Wrapping, ops::ControlFlow,
    sync::Arc, time::Duration,
};

use chrono::Utc;
use common::{DisconnectReason, MessagePayload, PlugMessage};
use dashmap::mapref::one::RefMut;
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use tokio::{
    net::ToSocketAddrs,
    select,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    time::timeout,
};
use tokio_util::udp::UdpFramed;
use tracing::{debug, info, warn};

use crate::{
    PlugCommand, PlugId, PlugTask, PowerState, SharedState, TaskRx,
    broker::proto::{BrokerCodec, CodecError},
};

mod proto;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    Working,
    Pinging([u8; 16]),
    Unknown,
    Dead,
}

type MsgTx = Sender<PlugMessage>;
type MsgRx = Receiver<PlugMessage>;

type BrokerSink = Arc<Mutex<SplitSink<UdpFramed<BrokerCodec>, (PlugMessage, SocketAddr)>>>;
type BrokerStream = SplitStream<UdpFramed<BrokerCodec>>;

pub struct Broker {
    stream: BrokerStream,
    sink: BrokerSink,
    shared_state: SharedState,
    sessions: HashMap<SocketAddr, MsgTx>,
}

struct BrokerConnection {
    sink: BrokerSink,
    addr: SocketAddr,
    /// Channel for protocol messages
    msg_rx: MsgRx,
    /// Channel for HTTP API tasks
    task_rx: Option<TaskRx>,
    /// HTTP API tasks
    tasks: Vec<PlugTask>,
    plug_id: Option<PlugId>,
    /// Holds shared state for plug power states and stuff
    shared_state: SharedState,
    /// Local message sequence number
    seq: Wrapping<u32>,
    /// Remote message sequence number
    client_seq: Wrapping<u32>,
    state: ConnectionState,
}

#[derive(Debug)]
enum ConnectionError {
    Dead,
    Codec(CodecError),
}

impl From<CodecError> for ConnectionError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

impl Broker {
    pub async fn new(addr: impl ToSocketAddrs, shared_state: SharedState) -> Self {
        let socket = tokio::net::UdpSocket::bind(addr).await.unwrap();
        let framed = UdpFramed::new(socket, BrokerCodec);
        let (sink, stream) = framed.split();

        Self {
            sink: Arc::new(Mutex::new(sink)),
            stream,
            shared_state,
            sessions: HashMap::new(),
        }
    }

    pub async fn run(&mut self) {
        tracing::info!("Broker initialized");

        loop {
            let (msg, addr) = self.stream.next().await.unwrap().unwrap();
            if matches!(msg.payload, MessagePayload::Conn { id: _ }) {
                let (tx, rx) = channel(16);
                tx.send(msg).await.unwrap();
                tracing::info!("New session for {addr}");
                self.sessions.insert(addr, tx);
                tokio::spawn(worker_task(
                    rx,
                    self.sink.clone(),
                    addr,
                    self.shared_state.clone(),
                ));
            } else if let Some(conn) = self.sessions.get(&addr) {
                conn.send(msg).await.unwrap();
            }
        }
    }
}

impl BrokerConnection {
    pub fn new(sink: BrokerSink, addr: SocketAddr, rx: MsgRx, shared_state: SharedState) -> Self {
        Self {
            sink,
            addr,
            msg_rx: rx,
            task_rx: None,
            tasks: Vec::default(),
            plug_id: None,
            shared_state,
            seq: Wrapping(rand::random()),
            client_seq: Wrapping(0),
            state: ConnectionState::Unknown,
        }
    }

    pub async fn send(&mut self, payload: MessagePayload) -> Result<(), ConnectionError> {
        self.seq += 1;
        self.sink
            .lock()
            .await
            .send((PlugMessage::new(self.seq.0, payload), self.addr))
            .await?;
        Ok(())
    }

    fn get_state_mut(&self) -> Option<RefMut<'_, PlugId, crate::PlugState>> {
        if let Some(id) = &self.plug_id {
            tracing::trace!("Locking state for {}", id.0);
            self.shared_state.plugs.get_mut(id)
        } else {
            None
        }
    }

    pub async fn disconnect(&mut self, reason: DisconnectReason) -> Result<(), ConnectionError> {
        self.state = ConnectionState::Dead;
        self.send(MessagePayload::Disconnect { reason }).await
    }

    pub fn can_recv(&self) -> bool {
        self.state != ConnectionState::Dead && !self.msg_rx.is_closed()
    }

    pub async fn recv(&mut self) -> Result<(), ConnectionError> {
        if !self.can_recv() {
            return Err(ConnectionError::Dead);
        }
        let next_task = async {
            if let Some(rx) = &mut self.task_rx {
                rx.recv().await
            } else {
                core::future::pending().await
            }
        };
        let next_msg = timeout(
            Duration::from_millis(rand::random_range(29000..=31000)),
            self.msg_rx.recv(),
        );

        let next = select! {
            msg = next_msg => {
                match msg {
                    Ok(Some(msg)) => {
                        if let Some(mut s) = self.get_state_mut() {
                            s.last_seen = Utc::now();
                        }
                        debug!("Received {msg:?} from {}", self.addr);
                        self.feed_msg(Some(msg)).await
                    },
                    Ok(None) => {
                        if let Some(mut s) = self.get_state_mut() {
                            s.last_seen = Utc::now();
                        }
                        warn!("Message pipe dead");
                        self.state = ConnectionState::Dead;
                        return Err(ConnectionError::Dead);
                    }
                    Err(_timeout) => {
                        info!("Message receiving timed out");
                        self.feed_msg(None).await
                    },
                }
            },
            task = next_task => {
                if let Some(task) = task {
                    tracing::debug!("Received new task {:?}", &task.command());
                    let cmd = task.command();
                    self.tasks.push(task);
                    match cmd {
                        PlugCommand::TurnOn => ControlFlow::Continue(Some(MessagePayload::TurnOn)),
                        PlugCommand::TurnOff => ControlFlow::Continue(Some(MessagePayload::TurnOff)),
                    }
                } else {
                    ControlFlow::Continue(None)
                }
            }
        };

        match next {
            ControlFlow::Continue(Some(msg)) => {
                debug!("Sending {msg:?} to {}", self.addr);
                self.send(msg).await
            }
            ControlFlow::Continue(None) => Ok(()),
            ControlFlow::Break(reason) => {
                info!("Disconnecting from {} ({reason:?})", self.addr);
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
            (Some(Mp::Conn { id }), ConnectionState::Unknown) => {
                let (tx, rx) = tokio::sync::mpsc::channel(4);
                self.shared_state.plugs.insert(
                    id.into(),
                    crate::PlugState {
                        last_seen: chrono::Utc::now(),
                        power_state: crate::PowerState::Unknown,
                        task_tx: tx,
                    },
                );
                self.plug_id = Some(id.into());
                tracing::info!("New plug connected: {id}");
                self.task_rx = Some(rx);

                self.client_seq = Wrapping(msg.unwrap().seq);
                ok!(Mp::ConnAck, Cs::Working)
            }
            (Some(Mp::Conn { id: _ }), _) => dc!(Dr::Closed),
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
                self.get_state_mut().unwrap().power_state = crate::PowerState::Off;
                for t in self
                    .tasks
                    .extract_if(.., |t| t.command() == PlugCommand::TurnOff)
                {
                    t.complete(true);
                }
                ok!()
            }
            (Some(Mp::TurnOnAck), _) => {
                self.get_state_mut().unwrap().power_state = crate::PowerState::On;
                for t in self
                    .tasks
                    .extract_if(.., |t| t.command() == PlugCommand::TurnOn)
                {
                    t.complete(true);
                }
                ok!()
            }
            (Some(Mp::StatusResp { is_on }), _) => {
                self.get_state_mut().unwrap().power_state = if is_on {
                    PowerState::On
                } else {
                    PowerState::Off
                };
                ok!()
            }
            (Some(m), Cs::Pinging(_d)) => {
                warn!("Unhandled message: {m:?}");
                ok!()
            }
            (Some(m), Cs::Working) => {
                warn!("Unhandled message: {m:?}");
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

async fn worker_task(rx: MsgRx, sink: BrokerSink, addr: SocketAddr, shared_state: SharedState) {
    let mut conn = BrokerConnection::new(sink, addr, rx, shared_state);
    loop {
        if let Err(e) = conn.recv().await {
            warn!("Connection with {addr} errored: {e}");
            break;
        }
    }
}

impl Display for ConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionError::Dead => f.write_str("Connection dead"),
            ConnectionError::Codec(codec_error) => write!(f, "{codec_error}"),
        }
    }
}

impl std::error::Error for ConnectionError {}

impl Drop for BrokerConnection {
    fn drop(&mut self) {
        if let Some(plug_id) = &self.plug_id {
            self.shared_state.plugs.remove(plug_id);
        }
    }
}
