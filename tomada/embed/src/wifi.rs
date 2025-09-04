use core::{net::Ipv4Addr, str::FromStr};

use alloc::string::String;
use common::{MessageGenerator, MessagePayload, PlugMessage};
use defmt::{debug, error, info, warn};
use dotenvy_macro::dotenv;
use embassy_futures::select::{Either, select, select3};
use embassy_net::{
    Config, DhcpConfig, IpListenEndpoint, Runner, Stack, StackResources, udp::PacketMetadata,
};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Instant, Timer, WithTimeout};
use esp_wifi::wifi::{
    ClientConfiguration, ScanConfig, WifiController, WifiDevice, WifiError, WifiEvent,
};
use futures::{FutureExt, future::join};

use crate::status_led::{LED_STATUS, LedStatusCode};

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
        info!("Starting controller");

        controller.is_started()?;

        for ap in controller
            .scan_with_config_async(ScanConfig {
                channel: None,
                ..Default::default()
            })
            .await?
        {
            defmt::debug!("Found AP ({}dBm): {}", ap.signal_strength, &*ap.ssid);
        }

        controller.set_configuration(&esp_wifi::wifi::Configuration::Client(
            ClientConfiguration {
                ssid: ssid.into(),
                password: passwd.map(|i| i.into()).unwrap_or_default(),
                ..Default::default()
            },
        ))?;

        info!("Connecting");
        controller.connect_async().await?;

        debug!("Waiting for connection up");
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
                defmt::error!("Wifi connection failed, retrying in {}ms: {}", delay, e);
                Timer::after_millis(delay).await;
                delay = core::cmp::min(delay * 2, 10000);
            }
            info!("Wifi connected");

            select(self.runner.lock().await.run(), async {
                info!("Turning on link");
                self.stack.wait_link_up().await;
                info!("Link up");

                info!("Configuring link");
                if self
                    .stack
                    .wait_config_up()
                    .with_timeout(Duration::from_secs(30))
                    .await
                    .is_err()
                {
                    warn!("DCHPv4 timed out while acquiring IP");
                    return;
                };
                info!("Link configured");

                match self.stack.config_v4() {
                    Some(c) => {
                        LED_STATUS.signal(LedStatusCode::Working);
                        info!("got IP from DHCP: {}", c.address);
                    }
                    None => {
                        LED_STATUS.signal(LedStatusCode::Disconnected);
                        error!("DHCPv4 returned no IP address");
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
                warn!("Wi-fi connection down. Restarting...");
                LED_STATUS.signal(LedStatusCode::Disconnected);
            })
            .await;
        }
    }
}

#[derive(Clone, Copy, defmt::Format)]
enum BrokerConnState {
    Working,
    PreConnect,
    Connecting,
    Disconnected,
    Pinging([u8; 8]),
}

const SEND_TIMEOUT: Duration = Duration::from_secs(5);

async fn broker_task(stack: Stack<'_>) -> ! {
    let mut state = BrokerConnState::PreConnect;

    macro_rules! send_error {
        ($send: expr) => {
            match $send {
                Ok(Ok(())) => debug!("Sent successfully"),
                Ok(Err(e)) => {
                    error!("Failed to send UDP socket info: {}", e);
                    state = BrokerConnState::Disconnected;
                }
                Err(_) => {
                    error!("UDP socket timed out");
                    state = BrokerConnState::Disconnected;
                }
            };
        };
        ($send: expr, $state: expr) => {
            match $send {
                Ok(Ok(())) => {
                    debug!("Sent successfully");
                    state = $state;
                }
                Ok(Err(e)) => {
                    error!("Failed to send UDP socket info: {}", e);
                    state = BrokerConnState::Disconnected;
                }
                Err(_) => {
                    error!("UDP socket timed out");
                    state = BrokerConnState::Disconnected;
                }
            };
        };
    }

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
    let mut msg_buf = [0u8; 256];

    sock.bind(IpListenEndpoint {
        addr: None,
        port: 4242,
    })
    .unwrap();

    let broker_addr = (
        Ipv4Addr::from_str(BROKER_IP).unwrap(),
        BROKER_PORT.parse::<u16>().unwrap(),
    );

    // TODO: random initial ID
    let mut generator = MessageGenerator::new(0);

    let mut send = async |msg: MessagePayload| {
        debug!("Sending {}", &msg);
        let msg = postcard::to_slice(&generator.new_message(msg), &mut msg_buf).unwrap();
        sock.send_to(msg, broker_addr)
            .with_timeout(SEND_TIMEOUT)
            .await
    };

    loop {
        if !stack.is_link_up() || !stack.is_config_up() {
            state = BrokerConnState::Disconnected;
            join(stack.wait_link_up(), stack.wait_config_up()).await;
        }
        let ping_timeout = Timer::at(Instant::now().checked_add(Duration::from_secs(30)).unwrap());

        info!("receiving");

        let mut buf = [0u8; 512];

        let r = select(ping_timeout, sock.recv_from(&mut buf)).await;

        match r {
            Either::First(_timeout) => {
                debug!("Sending ping");
                send_error!(
                    send(common::MessagePayload::Ping {
                        data: &[1, 2, 3, 4, 5, 6, 7, 8]
                    })
                    .await,
                    BrokerConnState::Pinging([1, 2, 3, 4, 5, 6, 7, 8])
                );
            }
            Either::Second(Ok(d)) => {
                let data = &buf[..d.0];
                let Ok(m) = postcard::from_bytes::<PlugMessage>(data) else {
                    warn!("Malformed postcard message");
                    continue;
                };
                info!("Received message: {}", m);
                match (m.payload, state) {
                    (MessagePayload::Ping { data }, _) => {
                        send_error!(send(MessagePayload::Pong { data }).await);
                    }
                    (MessagePayload::Pong { data }, BrokerConnState::Pinging(d)) => {
                        if d == data {
                            state = BrokerConnState::Working;
                        } else {
                            warn!("Pong data doesnt match!");
                        }
                    }
                    (MessagePayload::Pong { data: _ }, _) => {
                        warn!("Received pong while not waiting for it");
                    }
                    (MessagePayload::TurnOff, _) => {
                        // TODO: implement turning on/off
                        send_error!(send(MessagePayload::TurnOffAck).await);
                    }
                    (MessagePayload::TurnOn, _) => {
                        // TODO: implement turning on/off
                        send_error!(send(MessagePayload::TurnOnAck).await);
                    }
                    (MessagePayload::QueryStatus, _) => {
                        // TODO: implement status reading
                        send_error!(send(MessagePayload::StatusResp { is_on: true }).await);
                    }
                    (MessagePayload::ConnAck, BrokerConnState::Connecting) => {
                        state = BrokerConnState::Working
                    }
                    (MessagePayload::ConnAck, s) => {
                        warn!("Received ConnAck during innapropriate stage: {}", s);
                    }
                    (MessagePayload::TurnOffAck, _)
                    | (MessagePayload::TurnOnAck, _)
                    | (MessagePayload::Conn, _)
                    | (MessagePayload::StatusResp { is_on: _ }, _) => {
                        debug!("Received message meant for broker")
                    }
                }
            }
            Either::Second(Err(_)) => {
                error!("Message truncated, buffer too short!");
                state = BrokerConnState::Disconnected
            }
        }

        Timer::after_secs(5).await;
    }
}
