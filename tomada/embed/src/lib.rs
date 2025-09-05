#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::unnecessary_safety_doc)]
#![deny(clippy::missing_safety_doc)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]

#[cfg(not(feature = "ble"))]
use embassy_futures::select::select;
#[cfg(feature = "ble")]
use embassy_futures::select::select3;
use esp_hal::gpio::{Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin, Pull};

#[cfg(feature = "ble")]
use crate::ble::BleHost;
use crate::status_led::StatusLed;

#[cfg(feature = "ble")]
mod ble;
mod status_led;
mod wifi;

extern crate alloc;

#[cfg(feature = "ble")]
pub use ble::BleHandler;
pub use wifi::WifiHandler;

pub struct App<'a> {
    /// Dev board status LED
    status_led: StatusLed<'a>,
    /// Plug board status LED
    plug_led: Output<'a>,
    /// Relay on plug
    relay: Output<'a>,
    /// Button on plug
    button: Input<'a>,
    wifi: WifiHandler<'a>,
    #[cfg(feature = "ble")]
    ble: BleHandler<'a>,
}

impl<'a> App<'a> {
    pub fn new(
        onboard_led_pin: impl OutputPin + 'a,
        plug_led_pin: impl OutputPin + 'a,
        relay_pin: impl OutputPin + 'a,
        button_pin: impl InputPin + 'a,
        wifi_handler: WifiHandler<'a>,
        #[cfg(feature = "ble")] ble_handler: BleHandler<'a>,
    ) -> Self {
        let plug_led = Output::new(plug_led_pin, Level::Low, OutputConfig::default());
        let relay = Output::new(relay_pin, Level::Low, OutputConfig::default());
        let button = Input::new(button_pin, InputConfig::default().with_pull(Pull::None));

        let status_led = StatusLed::new(onboard_led_pin);

        Self {
            status_led,
            plug_led,
            relay,
            button,
            wifi: wifi_handler,
            #[cfg(feature = "ble")]
            ble: ble_handler,
        }
    }

    pub async fn run(#[allow(unused_mut)] mut self) -> ! {
        #[cfg(feature = "ble")]
        select3(self.wifi.run(), self.ble.run(), self.status_led.blink_led()).await;
        #[cfg(not(feature = "ble"))]
        select(self.wifi.run(), self.status_led.blink_led()).await;

        panic!("AAAA");
    }
}
