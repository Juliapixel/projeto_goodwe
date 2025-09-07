#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::unnecessary_safety_doc)]
#![deny(clippy::missing_safety_doc)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]

#[cfg(not(feature = "ble"))]
use embassy_futures::select::select5;
#[cfg(feature = "ble")]
use embassy_futures::select::select6;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use esp_hal::gpio::{Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin, Pull};

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
        select6(
            self.wifi.run(),
            self.ble.run(),
            self.status_led.blink_led(),
            led_task(&mut self.plug_led),
            relay_task(&mut self.relay),
            button_task(&mut self.button),
        )
        .await;
        #[cfg(not(feature = "ble"))]
        select5(
            self.wifi.run(),
            self.status_led.blink_led(),
            led_task(&mut self.plug_led),
            relay_task(&mut self.relay),
            button_task(&mut self.button),
        )
        .await;

        panic!("AAAA");
    }
}

pub type PinSignal = Signal<CriticalSectionRawMutex, Level>;
pub type PinStatus = Watch<CriticalSectionRawMutex, Level, 4>;

pub static RELAY_SIGNAL: PinSignal = Signal::new();
pub static RELAY_STATUS: PinStatus = Watch::new();

async fn relay_task(pin: &mut Output<'_>) {
    let orig_level = pin.output_level();
    let should_signal = RELAY_STATUS.try_get().is_none_or(|l| l == orig_level);
    let sender = RELAY_STATUS.sender();
    if should_signal {
        sender.send(orig_level);
    }
    loop {
        let level = RELAY_SIGNAL.wait().await;
        if level != pin.output_level() {
            pin.set_level(level);
            sender.send(level);
        }
    }
}

pub static PLUG_LED_SIGNAL: PinSignal = Signal::new();
pub static PLUG_LED_STATUS: PinStatus = Watch::new();

async fn led_task(pin: &mut Output<'_>) {
    let orig_level = pin.output_level();
    let should_signal = PLUG_LED_STATUS.try_get().is_none_or(|l| l == orig_level);
    let sender = PLUG_LED_STATUS.sender();
    if should_signal {
        sender.send(orig_level);
    }
    loop {
        let level = PLUG_LED_SIGNAL.wait().await;
        if level != pin.output_level() {
            pin.set_level(level);
            sender.send(level);
        }
    }
}

pub static BUTTON_STATUS: PinStatus = Watch::new();

async fn button_task(_pin: &mut Input<'_>) {
    // TODO: detect plug button presses
    core::future::pending::<()>().await;
}
