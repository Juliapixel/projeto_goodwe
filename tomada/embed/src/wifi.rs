use core::{
    net::{Ipv4Addr, SocketAddrV4},
    num::Wrapping,
    ops::ControlFlow,
    str::FromStr,
};

use alloc::string::String;
use common::{DisconnectReason, MessagePayload, PlugMessage};
use defmt::{debug, error, info, warn};
use dotenvy_macro::dotenv;
use embassy_futures::select::{select, select3};
use embassy_net::{
    Config, DhcpConfig, IpListenEndpoint, Runner, Stack, StackResources,
    udp::{PacketMetadata, RecvError, SendError, UdpSocket},
};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    mutex::Mutex,
    watch::Receiver,
};
use embassy_time::{Duration, TimeoutError, Timer, WithTimeout};
use esp_hal::gpio::Level;
use esp_wifi::wifi::{
    ClientConfiguration, ScanConfig, WifiController, WifiDevice, WifiError, WifiEvent,
};
use futures::FutureExt;

use crate::{
    RELAY_SIGNAL, RELAY_STATUS,
    status_led::{LED_STATUS, LedStatusCode},
};

extern crate alloc;

pub struct WifiHandler<'a> {
    controller: Mutex<NoopRawMutex, WifiController<'a>>,
    stack: Stack<'a>,
    runner: Mutex<NoopRawMutex, Runner<'a, WifiDevice<'a>>>,
}

const BROKER_IP: &str = dotenv!("BROKER_IP");
const BROKER_PORT: &str = dotenv!("BROKER_PORT");

const SSID: &str = dotenv!("SSID");
const PASSWORD: &str = dotenv!("PASSWORD");

impl<'a> WifiHandler<'a> {
    pub fn new(
        controller: WifiController<'a>,
        device: WifiDevice<'a>,
        stack_resources: &'a mut StackResources<5>,
        seed: u64,
    ) -> Self {
        let (stack, runner) = embassy_net::new(
            device,
            Config::dhcpv4(DhcpConfig::default()),
            stack_resources,
            seed,
        );

        Self {
            controller: Mutex::new(controller),
            stack,
            runner: Mutex::new(runner),
        }
    }

    pub async fn connect(
        &self,
        ssid: impl Into<String>,
        passwd: Option<impl Into<String>>,
    ) -> Result<(), WifiError> {
        let mut controller = self.controller.lock().await;

        controller.set_mode(esp_wifi::wifi::WifiMode::Sta)?;
        controller.set_power_saving(esp_wifi::config::PowerSaveMode::None)?;

        if controller.is_connected().is_ok_and(|i| i) {
            controller.disconnect_async().await?;
        }

        if controller.is_started()? {
            controller.stop_async().await?;
        }

        controller.start_async().await?;
        info!("[wifi] Starting controller");

        controller.is_started()?;

        for ap in controller
            .scan_with_config_async(ScanConfig {
                channel: None,
                ..Default::default()
            })
            .await?
        {
            defmt::debug!("[wifi] Found AP ({}dBm): {}", ap.signal_strength, &*ap.ssid);
        }

        controller.set_configuration(&esp_wifi::wifi::Configuration::Client(
            ClientConfiguration {
                ssid: ssid.into(),
                password: passwd.map(|i| i.into()).unwrap_or_default(),
                ..Default::default()
            },
        ))?;

        info!("[wifi] Connecting");
        controller.connect_async().await?;

        debug!("[wifi] Waiting for connection up");
        loop {
            if controller.is_connected()? {
                return Ok(());
            }
        }
    }

    pub async fn run(&self) {
        LED_STATUS.signal(LedStatusCode::Connecting);
        loop {
            let mut delay = 1000;
            while let Err(e) = self.connect(SSID, Some(PASSWORD)).await {
                defmt::error!("[wifi] Connection failed, retrying in {}ms: {}", delay, e);
                Timer::after_millis(delay).await;
                delay = core::cmp::min(delay * 2, 10000);
            }
            info!("[wifi] Connected");

            select(self.runner.lock().await.run(), async {
                info!("[wifi] Turning on link");
                self.stack.wait_link_up().await;
                info!("[wifi] Link up");

                info!("[wifi] Configuring link");
                if self
                    .stack
                    .wait_config_up()
                    .with_timeout(Duration::from_secs(30))
                    .await
                    .is_err()
                {
                    warn!("[wifi] DCHPv4 timed out while acquiring IP");
                    return;
                };
                info!("[wifi] Link configured");

                match self.stack.config_v4() {
                    Some(c) => {
                        LED_STATUS.signal(LedStatusCode::Working);
                        info!("[wifi] got IP from DHCP: {}", c.address);
                    }
                    None => {
                        LED_STATUS.signal(LedStatusCode::Disconnected);
                        error!("[wifi] DHCPv4 returned no IP address");
                        return;
                    }
                }

                select3(
                    broker_task(self.stack),
                    self.stack.wait_link_down(),
                    self.controller.lock().then(async |mut c| {
                        c.wait_for_events([WifiEvent::StaDisconnected].into(), false)
                            .await
                    }),
                )
                .await;
                warn!("[wifi] Connection down. Restarting...");
                LED_STATUS.signal(LedStatusCode::Disconnected);
            })
            .await;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, defmt::Format)]
enum ConnState {
    Working,
    Connecting,
    Disconnected,
    Pinging([u8; 16]),
}

const SEND_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, defmt::Format)]
enum ConnError {
    NoRoute,
    PacketTooLarge,
    SocketNotBound,
    Postcard(postcard::Error),
    SendTimeout,
    RecvBufferTooSmall,
}

impl From<postcard::Error> for ConnError {
    fn from(value: postcard::Error) -> Self {
        Self::Postcard(value)
    }
}

impl From<SendError> for ConnError {
    fn from(value: SendError) -> Self {
        match value {
            SendError::NoRoute => Self::NoRoute,
            SendError::SocketNotBound => Self::SocketNotBound,
            SendError::PacketTooLarge => Self::PacketTooLarge,
        }
    }
}

impl From<RecvError> for ConnError {
    fn from(_value: RecvError) -> Self {
        Self::RecvBufferTooSmall
    }
}

impl From<TimeoutError> for ConnError {
    fn from(_value: TimeoutError) -> Self {
        Self::SendTimeout
    }
}

struct Client<'a> {
    seq: Wrapping<u32>,
    server_seq: Wrapping<u32>,
    state: ConnState,
    addr: SocketAddrV4,
    socket: UdpSocket<'a>,
    relay_state: Receiver<'static, CriticalSectionRawMutex, Level, 4>,
}

impl<'a> Client<'a> {
    fn new(addr: SocketAddrV4, socket: UdpSocket<'a>) -> Self {
        Self {
            state: ConnState::Disconnected,
            addr,
            socket,
            seq: Default::default(),
            server_seq: Default::default(),
            relay_state: RELAY_STATUS.receiver().unwrap(),
        }
    }

    pub async fn send(&mut self, msg: MessagePayload) -> Result<(), ConnError> {
        let mut buf = [0u8; 256];
        self.seq += 1;
        debug!("[broker] Sending message: {}", msg);
        self.socket
            .send_to(
                postcard::to_slice(&PlugMessage::new(self.seq.0, msg), &mut buf).unwrap(),
                (*self.addr.ip(), self.addr.port()),
            )
            .with_timeout(SEND_TIMEOUT)
            .await??;
        Ok(())
    }

    pub async fn connect(&mut self) -> Result<(), ConnError> {
        self.state = ConnState::Connecting;
        self.send(MessagePayload::Conn).await
    }

    pub async fn disconnect(&mut self, reason: DisconnectReason) -> Result<(), ConnError> {
        self.state = ConnState::Disconnected;
        self.send(MessagePayload::Disconnect { reason }).await
    }

    pub async fn recv(&mut self) {
        let mut buf = [0u8; 512];
        let rcv = self
            .socket
            .recv_from(&mut buf)
            .with_timeout(Duration::from_secs(30))
            .await;
        let rcv = rcv.map(|r| r.map(|m| postcard::from_bytes::<PlugMessage>(&buf[..m.0])));
        let msg = match rcv {
            Ok(Ok(Ok(msg))) => {
                debug!("[broker] Received message: {}", msg);
                self.feed_msg(Some(msg))
            }
            Ok(Ok(Err(_pe))) => ControlFlow::Break(DisconnectReason::ProtocolError),
            Ok(Err(_se)) => {
                error!("[broker] Rx buffer too small");
                ControlFlow::Break(DisconnectReason::Closed)
            }
            Err(_timeout) => self.feed_msg(None),
        };

        match msg {
            ControlFlow::Continue(Some(msg)) => {
                if let Err(e) = self.send(msg).await {
                    error!("[broker] Sending to socket failed: {}", e);
                }
            }
            ControlFlow::Continue(None) => (),
            ControlFlow::Break(reason) => {
                warn!("[broker] Requested disconnect: {}", reason);
                if let Err(e) = self.disconnect(reason).await {
                    warn!("[broker] Sending Disconnect to socket failed: {}", e);
                }
            }
        }
    }

    /// Takes a [PlugMessage](crate::PlugMessage), updates internal state and
    /// returns the desired response
    ///
    /// # Params
    /// - `msg`: Must be `None` if receiving the message timed out
    ///   and a heartbeat should be sent
    pub fn feed_msg(
        &mut self,
        msg: Option<PlugMessage>,
    ) -> ControlFlow<DisconnectReason, Option<MessagePayload>> {
        use ConnState as S;
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
            && self.state != S::Connecting
        {
            if msg.seq != (self.server_seq + Wrapping(1)).0 {
                return ControlFlow::Break(Dr::SequenceError);
            } else {
                self.server_seq = Wrapping(msg.seq);
            }
        }

        match (msg.map(|m| m.payload), self.state) {
            (Some(Mp::ConnAck), S::Connecting) => {
                self.server_seq = Wrapping(msg.unwrap().seq);
                ok!(, S::Working)
            }
            (Some(Mp::ConnAck), _) => dc!(Dr::Closed),
            (Some(Mp::Disconnect { reason }), _) => {
                warn!("[broker] Server requested disconnect: {:?}", reason);
                dc!(Dr::Closed)
            }
            (Some(Mp::Ping { data }), _) => ok!(Mp::Pong { data }),
            (Some(Mp::Pong { data }), S::Pinging(d)) => {
                if data == d {
                    ok!(, S::Working)
                } else {
                    dc!(Dr::BadHeartbeat)
                }
            }
            (Some(Mp::Pong { data: _ }), _) => dc!(Dr::ProtocolError),
            (Some(Mp::TurnOff), _) => {
                info!("[broker] Broker requested TurnOff");
                RELAY_SIGNAL.signal(Level::Low);
                ok!(Mp::TurnOnAck)
            }
            (Some(Mp::TurnOn), _) => {
                info!("[broker] Broker requested TurnOn");
                RELAY_SIGNAL.signal(Level::High);
                ok!(Mp::TurnOnAck)
            }
            (Some(Mp::QueryStatus), _) => {
                let is_on = self.relay_state.try_get().is_some_and(|l| l == Level::High);
                ok!(Mp::StatusResp { is_on })
            }
            (Some(m), S::Working) => {
                info!("[broker] Unhandled message: {:?}", m);
                ok!()
            }
            (None, S::Pinging(_)) => dc!(Dr::Timeout),
            (None, S::Working) => {
                let mut data = [0u8; 16];
                data.fill(42);
                // TODO: random ping data
                // self.rng.fill_bytes(&mut data);
                ok!(Mp::Ping { data }, S::Pinging(data))
            }
            (None, S::Connecting) => ok!(, S::Disconnected),
            (None, _) => dc!(Dr::Closed),
            (_, S::Connecting) => dc!(Dr::Closed),
            (_, _) => ok!(),
        }
    }

    pub async fn run(&mut self) -> ! {
        loop {
            if self.state == ConnState::Disconnected
                && let Err(e) = self.connect().await
            {
                warn!("[broker] Failed to send connection request: {}", e);
            }
            self.recv().await;
        }
    }
}

async fn broker_task(stack: Stack<'_>) -> ! {
    let mut tx_meta = [PacketMetadata::EMPTY; 4];
    let mut tx_buf = [0u8; 1024];
    let mut rx_meta = [PacketMetadata::EMPTY; 4];
    let mut rx_buf = [0u8; 1024];

    let mut sock = embassy_net::udp::UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buf,
        &mut tx_meta,
        &mut tx_buf,
    );

    sock.bind(IpListenEndpoint {
        addr: None,
        port: 4242,
    })
    .unwrap();

    let broker_ip = Ipv4Addr::from_str(BROKER_IP).unwrap();
    let broker_port = BROKER_PORT.parse::<u16>().unwrap();

    let mut client = Client::new(SocketAddrV4::new(broker_ip, broker_port), sock);
    client.run().await;
}
