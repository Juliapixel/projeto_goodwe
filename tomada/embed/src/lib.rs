#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::unnecessary_safety_doc)]
#![deny(clippy::missing_safety_doc)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]

use alloc::rc::Rc;
use defmt::info;
use dotenvy_macro::dotenv;
#[cfg(not(feature = "ble"))]
use embassy_futures::select::select;
use embassy_futures::select::select3;
use embassy_net::StackResources;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::Timer;
use esp_hal::gpio::{Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin, Pull};
use esp_wifi::wifi::{WifiController, WifiDevice};

#[cfg(feature = "ble")]
use crate::ble::{BleHandler, BleHost};
use crate::{
    status_led::{LedStatusCode, StatusLed},
    wifi::WifiHandler,
};

#[cfg(feature = "ble")]
mod ble;
mod status_led;
mod wifi;

extern crate alloc;

pub struct App<'a> {
    status_led: StatusLed<'a>,
    status_led_signal: Rc<Signal<NoopRawMutex, LedStatusCode>>,
    plug_led: Output<'a>,
    relay: Output<'a>,
    button: Input<'a>,
    wifi: WifiHandler<'a>,
    #[cfg(feature = "ble")]
    ble: BleHandler<'a>,
}

const SSID: &str = dotenv!("SSID");
const PASSWORD: &str = dotenv!("PASSWORD");

impl<'a> App<'a> {
    pub fn new(
        onboard_led_pin: impl OutputPin + 'a,
        plug_led_pin: impl OutputPin + 'a,
        relay_pin: impl OutputPin + 'a,
        button_pin: impl InputPin + 'a,
        controller: WifiController<'static>,
        mut device: WifiDevice<'static>,
        stack_resources: &'a mut StackResources<5>,
        #[cfg(feature = "ble")] ble_host: BleHost<'a>,
        seed: u64,
    ) -> Self {
        let plug_led = Output::new(plug_led_pin, Level::Low, OutputConfig::default());
        let relay = Output::new(relay_pin, Level::Low, OutputConfig::default());
        let button = Input::new(button_pin, InputConfig::default().with_pull(Pull::None));

        let (status_led, led_signal) = StatusLed::new(onboard_led_pin);

        Self {
            status_led,
            status_led_signal: led_signal,
            plug_led,
            relay,
            button,
            wifi: WifiHandler::new(controller, device, stack_resources, seed),
            #[cfg(feature = "ble")]
            ble: BleHandler::new(ble_host),
        }
    }

    pub async fn run(mut self) -> ! {
        while let Err(e) = self.wifi.connect(SSID, Some(PASSWORD)).await {
            defmt::error!("{}", e);
            Timer::after_millis(1000).await;
        }
        info!("Wifi connected");

        self.status_led_signal.signal(LedStatusCode::Connecting);

        #[cfg(feature = "ble")]
        select3(
            self.wifi.run(&self.status_led_signal),
            self.ble.run(),
            self.status_led.blink_led(),
        )
        .await;
        #[cfg(not(feature = "ble"))]
        select(
            self.wifi.run(&self.status_led_signal),
            self.status_led.blink_led(),
        )
        .await;

        panic!("AAAA");
    }
}
