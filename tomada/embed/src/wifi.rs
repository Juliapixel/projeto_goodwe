use core::{net::Ipv4Addr, pin::pin, str::FromStr};

use alloc::string::String;
use defmt::{error, info, warn};
use dotenvy_macro::dotenv;
use embassy_net::{udp::PacketMetadata, Config, DhcpConfig, IpListenEndpoint, Runner, Stack, StackResources};
use embassy_sync::{blocking_mutex::raw::{NoopRawMutex, RawMutex}, mutex::Mutex, signal::Signal};
use embassy_time::{Duration, Timer, WithTimeout};
use esp_wifi::wifi::{ClientConfiguration, WifiController, WifiDevice, WifiError, WifiEvent};
use futures::future::{join, select};

use crate::status_led::LedStatusCode;

extern crate alloc;

pub struct WifiHandler<'a> {
    controller: Mutex<NoopRawMutex, WifiController<'a>>,
    stack: Stack<'a>,
    runner: Mutex<NoopRawMutex, Runner<'a, WifiDevice<'a>>>,
}

pub enum WifiCommand {
    SetWifi {
        ssid: String,
        password: Option<String>
    },
    Reconnect,
    Disconnect,
}

const BROKER_IP: &str = dotenv!("BROKER_IP");
const BROKER_PORT: &str = dotenv!("BROKER_PORT");

impl<'a> WifiHandler<'a> {
    pub fn new(
        controller: WifiController<'a>,
        device: WifiDevice<'a>,
        stack_resources: &'a mut StackResources<5>,
        seed: u64
    ) -> Self {

        let (stack, runner) = embassy_net::new(
            device,
            Config::dhcpv4(DhcpConfig::default()),
            stack_resources,
            seed
        );

        Self { controller: Mutex::new(controller), stack, runner: Mutex::new(runner) }
    }

    pub async fn disconnect(&self) -> Result<(), WifiError> {
        let mut ctrl = self.controller.lock().await;
        if ctrl.is_connected()? {
            ctrl.disconnect_async().await?;
        }
        if ctrl.is_started()? {
            ctrl.stop_async().await?;
        }
        Ok(())
    }

    pub async fn connect(
        &self,
        ssid: impl Into<String>,
        passwd: Option<impl Into<String>>,
    ) -> Result<(), WifiError> {
        let mut controller = self.controller.lock().await;

        controller.set_mode(esp_wifi::wifi::WifiMode::Sta)?;
        controller.set_power_saving(esp_wifi::config::PowerSaveMode::None)?;

        if controller.is_connected()? {
            controller.disconnect_async().await?;
        }

        if controller.is_started()? {
            controller.stop_async().await?;
        }

        controller.start_async().await?;
        info!("Starting controller");

        controller.is_started()?;

        if cfg!(debug_assertions) {
            for ap in controller.scan_n_async(10).await? {
                defmt::debug!("Found AP: {}", &*ap.ssid);
                defmt::debug!("Auth method: {}", ap.auth_method);
                defmt::debug!("Signal strength: {}dBm", ap.signal_strength);
            }
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

        loop {
            if controller.is_connected()? {
                return Ok(());
            }
        }
    }

    async fn reconnect(&self) -> Result<(), WifiError> {
        self.disconnect().await?;
        self.controller.lock().await.connect_async().await
    }

    pub async fn run(&self, led_signal: &Signal<impl RawMutex, LedStatusCode>) {
        join(
            pin!(self.runner.lock().await.run()),
            pin!(async { loop {
                led_signal.signal(LedStatusCode::Connecting);
                info!("Turning on link");
                self.stack.wait_link_up().await;
                info!("Link up");

                info!("Configuring link");
                self.stack.wait_config_up().await;
                info!("Link configured");

                match self.stack.config_v4() {
                    Some(c) => {
                        info!("got ip from dhcp: {}", c.address);
                    }
                    None => {
                        error!("no ip saj");
                    }
                }
                led_signal.signal(LedStatusCode::Working);

                broker_task(self.stack).await;

                select(
                    pin!(self.stack.wait_link_down()),
                    pin!(self.controller.lock().await.wait_for_events([WifiEvent::StaDisconnected, WifiEvent::StaBeaconTimeout].into(), false))
                ).await;
                warn!("Wi-fi connection down. Restarting...");
                led_signal.signal(LedStatusCode::Disconnected);
                self.reconnect().await;
            }})
        ).await;
    }
}

async fn broker_task(stack: Stack<'_>) {
    let mut tx_meta = [PacketMetadata::EMPTY; 4];
    let mut tx_buf = [0u8; 1024];
    let mut rx_meta = [PacketMetadata::EMPTY; 4];
    let mut rx_buf = [0u8; 1024];

    let mut sock = embassy_net::udp::UdpSocket::new(stack, &mut rx_meta, &mut rx_buf, &mut tx_meta, &mut tx_buf);
    let mut ping = [0u8;256];
    let ping = postcard::to_slice(&common::PlugMessage::Ping { data: &[1,2,3,4,5,6] }, &mut ping).unwrap();

    sock.bind(IpListenEndpoint {
        addr: None,
        port: 4242
    }).unwrap();

    loop {
        let send = sock.send_to(
            ping,
            (Ipv4Addr::from_str(BROKER_IP).unwrap(), BROKER_PORT.parse().unwrap())
        ).with_timeout(Duration::from_secs(5)).await;

        match send {
            Ok(Ok(_)) => (),
            Ok(Err(e)) => {error!("Failed to send UDP socket info: {}", e); continue;},
            Err(_) => {error!("UDP socket timed out"); continue;}
        };

        info!("receiving");
        sock.recv_from_with(|d, _m| {
            info!(
                "{}",
                postcard::from_bytes::<common::PlugMessage>(d)
            );
        }).with_timeout(Duration::from_secs(5)).await;

        Timer::after_secs(5).await;
    }

}
