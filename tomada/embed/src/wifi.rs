use core::{
    net::{Ipv4Addr, SocketAddrV4},
    num::Wrapping,
    ops::{ControlFlow, Deref},
    str::FromStr,
};

use crate::{debug, error, info, warn};
use alloc::string::String;
use common::{DisconnectReason, MessagePayload, PlugMessage};
use dotenvy_macro::{dotenv, option_dotenv};
use embassy_futures::select::{Either, select, select3};
use embassy_net::{
    Config, DhcpConfig, IpAddress, IpListenEndpoint, Runner, Stack, StackResources,
    udp::{PacketMetadata, RecvError, SendError, UdpSocket},
};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    lazy_lock::LazyLock,
    mutex::Mutex,
    watch::{Receiver, Watch},
};
use embassy_time::{Duration, TimeoutError, Timer, WithTimeout};
use esp_wifi::wifi::{
    ClientConfiguration, ScanConfig, WifiController, WifiDevice, WifiError, WifiEvent,
};
use futures::FutureExt;

use crate::{
    RELAY_SIGNAL, RELAY_STATUS, RelayMode,
    status_led::{LED_STATUS, LedStatusCode},
};

extern crate alloc;

pub struct WifiHandler<'a> {
    controller: Mutex<NoopRawMutex, WifiController<'a>>,
    stack: Stack<'a>,
    runner: Mutex<NoopRawMutex, Runner<'a, WifiDevice<'a>>>,
}

const BROKER_IP: Option<&str> = option_dotenv!("BROKER_IP");
const BROKER_HOST: &str = match option_dotenv!("BROKER_HOST") {
    Some(host) => host,
    None => "goodwe.juliapixel.com",
};
const BROKER_PORT: u16 = {
    match u16::from_str_radix(dotenv!("BROKER_PORT"), 10) {
        Ok(port) => port,
        Err(_) => panic!("BROKER_PORT env variable must be a u16"),
    }
};

const SSID_PASSWORD: (&str, &str) = (dotenv!("SSID"), dotenv!("PASSWORD"));

const SSID_PASSWORD2: Option<(&str, &str)> =
    match (option_dotenv!("SSID2"), option_dotenv!("PASSWORD2")) {
        (Some(s), Some(p)) => Some((s, p)),
        (None, None) => None,
        _ => panic!(),
    };

const SSID_PASSWORD3: Option<(&str, &str)> =
    match (option_dotenv!("SSID3"), option_dotenv!("PASSWORD3")) {
        (Some(s), Some(p)) => Some((s, p)),
        (None, None) => None,
        _ => panic!(),
    };

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
        ssid: impl Into<String> + Deref<Target = str>,
        passwd: Option<impl Into<String> + Deref<Target = str>>,
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

        // Safety: safe to call with valid value 8
        // idk where i saw this but thank you to whoever suggested lowering tx power
        unsafe {
            esp_wifi_sys::include::esp_wifi_set_max_tx_power(8);
        }

        controller.is_started()?;

        for ap in controller
            .scan_with_config_async(ScanConfig {
                channel: None,
                ..Default::default()
            })
            .await?
        {
            debug!("[wifi] Found AP ({}dBm): {}", ap.signal_strength, &*ap.ssid);
        }

        info!("[wifi] Connecting to {}", &*ssid);

        controller.set_configuration(&esp_wifi::wifi::Configuration::Client(
            ClientConfiguration {
                ssid: ssid.into(),
                password: passwd.map(|i| i.into()).unwrap_or_default(),
                ..Default::default()
            },
        ))?;

        controller.connect_async().await?;

        debug!("[wifi] Waiting for connection up");
        loop {
            if controller.is_connected()? {
                return Ok(());
            } else {
                let _ = controller
                    .wait_for_events(
                        [WifiEvent::StaConnected, WifiEvent::StaDisconnected].into(),
                        false,
                    )
                    .with_timeout(Duration::from_secs(1))
                    .await;
            }
        }
    }

    pub async fn connect_many<S, P>(
        &self,
        pairs: impl IntoIterator<Item = (S, P)>,
    ) -> Result<(), WifiError>
    where
        S: Into<String> + Deref<Target = str>,
        P: Into<String> + Deref<Target = str>,
    {
        for (ssid, passwd) in pairs {
            if let Err(e) = self.connect(ssid, Some(passwd)).await {
                if !matches!(e, WifiError::Disconnected) {
                    return Err(e);
                }
            } else {
                return Ok(());
            }
        }
        Err(WifiError::Disconnected)
    }

    pub async fn run(&self) {
        LED_STATUS.signal(LedStatusCode::Connecting);
        loop {
            let mut delay = 1000;

            while let Err(e) = self
                .connect_many(
                    [Some(SSID_PASSWORD), SSID_PASSWORD2, SSID_PASSWORD3]
                        .into_iter()
                        .flatten(),
                )
                .await
            {
                error!("[wifi] Connection failed, retrying in {}ms: {}", delay, e);
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

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum ConnState {
    Working,
    Connecting,
    Disconnected,
    Pinging([u8; 16]),
}

const SEND_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
    relay_state: Receiver<'static, CriticalSectionRawMutex, RelayMode, 4>,
}

pub static WIFI_MSG_CHANNEL: Watch<CriticalSectionRawMutex, MessagePayload, 1> = Watch::new();

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
        // TODO: random UUID
        static UUID: LazyLock<uuid::Uuid> =
            LazyLock::new(|| "338c1c8a-c3a2-4715-be92-8911248bbb8c".parse().unwrap());
        self.state = ConnState::Connecting;
        self.send(MessagePayload::Conn { id: *UUID.get() }).await
    }

    pub async fn disconnect(&mut self, reason: DisconnectReason) -> Result<(), ConnError> {
        self.state = ConnState::Disconnected;
        self.send(MessagePayload::Disconnect { reason }).await
    }

    pub async fn recv(&mut self) -> ControlFlow<DisconnectReason, Option<MessagePayload>> {
        let mut buf = [0u8; 512];
        let rcv = self
            .socket
            .recv_from(&mut buf)
            .with_timeout(Duration::from_secs(30))
            .await;
        let rcv = rcv.map(|r| r.map(|m| postcard::from_bytes::<PlugMessage>(&buf[..m.0])));
        match rcv {
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
        }
    }

    pub async fn send_response(
        &mut self,
        msg: ControlFlow<DisconnectReason, Option<MessagePayload>>,
    ) {
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
                RELAY_SIGNAL.signal(RelayMode::Open);
                ok!(Mp::TurnOffAck)
            }
            (Some(Mp::TurnOn), _) => {
                info!("[broker] Broker requested TurnOn");
                RELAY_SIGNAL.signal(RelayMode::Closed);
                ok!(Mp::TurnOnAck)
            }
            (Some(Mp::QueryStatus), _) => {
                let is_on = self
                    .relay_state
                    .try_get()
                    .is_some_and(|l| l == RelayMode::Closed);
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
        let mut receiver = WIFI_MSG_CHANNEL.receiver().unwrap();
        loop {
            if self.state == ConnState::Disconnected
                && let Err(e) = self.connect().await
            {
                warn!("[broker] Failed to send connection request: {}", e);
            }
            match select(self.recv(), receiver.changed()).await {
                Either::First(f) => self.send_response(f).await,
                Either::Second(s) => {
                    let _ = self.send(s).await;
                }
            }
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

    let broker_ip = match stack
        .dns_query(BROKER_HOST, smoltcp::wire::DnsQueryType::A)
        .await
    {
        Ok(a) => {
            debug!("[broker] got broker IP from DNS: {}", &a);
            let addr = a.first().copied().map(|a| {
                let IpAddress::Ipv4(addr) = a;
                addr
            });
            if addr.is_none() {
                warn!("[broker] DNS A query for {} returned no IPs", BROKER_HOST);
            }
            addr
        }
        Err(e) => {
            error!("[broker] DNS query failed: {}", e);
            None
        }
    }
    .or_else(|| BROKER_IP.and_then(|ip| Ipv4Addr::from_str(ip).ok()));

    info!(
        "[broker] Starting new connection to {}:{}",
        broker_ip, BROKER_PORT
    );

    if let Some(ip) = broker_ip {
        let mut client = Client::new(SocketAddrV4::new(ip, BROKER_PORT), sock);
        client.run().await;
    } else {
        error!("[broker] no IP available for the message broker.");
        #[allow(clippy::empty_loop)]
        core::future::pending::<()>().then(async |_| loop {}).await
    }
}
