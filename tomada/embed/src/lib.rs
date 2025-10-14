#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::multiple_unsafe_ops_per_block)]
#![deny(clippy::undocumented_unsafe_blocks)]
#![deny(clippy::unnecessary_safety_doc)]
#![deny(clippy::missing_safety_doc)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]

use common::MessagePayload;
#[cfg(not(feature = "ble"))]
use embassy_futures::select::select4;
#[cfg(feature = "ble")]
use embassy_futures::select::select5;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal, watch::Watch};
use embassy_time::Timer;
use esp_hal::gpio::{Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin, Pull};

use crate::{status_led::StatusLed, wifi::WIFI_MSG_CHANNEL};

#[cfg(feature = "ble")]
mod ble;
mod fmt;
mod status_led;
mod wifi;

extern crate alloc;

#[cfg(feature = "ble")]
pub use ble::BleHandler;
pub use fmt::*;
pub use wifi::WifiHandler;

pub struct App<'a> {
    /// Status LED handler
    status_led: StatusLed<'a>,
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
        let relay = Output::new(relay_pin, RelayMode::Closed.into(), OutputConfig::default());
        let button = Input::new(button_pin, InputConfig::default().with_pull(Pull::None));

        let status_led = StatusLed::new(onboard_led_pin, plug_led_pin);

        Self {
            status_led,
            relay,
            button,
            wifi: wifi_handler,
            #[cfg(feature = "ble")]
            ble: ble_handler,
        }
    }

    pub async fn run(#[allow(unused_mut)] mut self) -> ! {
        #[cfg(feature = "ble")]
        select5(
            self.wifi.run(),
            self.ble.run(),
            self.status_led.run(),
            relay_task(&mut self.relay),
            button_task(&mut self.button),
        )
        .await;
        #[cfg(not(feature = "ble"))]
        select4(
            self.wifi.run(),
            self.status_led.run(),
            relay_task(&mut self.relay),
            button_task(&mut self.button),
        )
        .await;

        panic!("AAAA");
    }
}

pub type PinSignal<T> = Signal<CriticalSectionRawMutex, T>;
pub type PinStatus<T> = Watch<CriticalSectionRawMutex, T, 4>;

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RelayMode {
    Open,
    Closed,
}

impl From<RelayMode> for Level {
    fn from(value: RelayMode) -> Self {
        match value {
            RelayMode::Closed => Level::High,
            RelayMode::Open => Level::Low,
        }
    }
}

impl From<Level> for RelayMode {
    fn from(value: Level) -> Self {
        match value {
            Level::Low => RelayMode::Open,
            Level::High => RelayMode::Closed,
        }
    }
}

impl RelayMode {
    fn toggle(&self) -> Self {
        match self {
            RelayMode::Open => RelayMode::Closed,
            RelayMode::Closed => RelayMode::Open,
        }
    }
}

pub static RELAY_SIGNAL: PinSignal<RelayMode> = Signal::new();
pub static RELAY_STATUS: PinStatus<RelayMode> = Watch::new();

async fn relay_task(pin: &mut Output<'_>) {
    let orig_level = pin.output_level();
    let should_signal = RELAY_STATUS
        .try_get()
        .is_none_or(|l| l == orig_level.into());
    let sender = RELAY_STATUS.sender();
    if should_signal {
        sender.send(orig_level.into());
    }
    loop {
        let mode = RELAY_SIGNAL.wait().await;
        if mode != pin.output_level().into() {
            pin.set_level(mode.into());
            sender.send(mode);
            match mode {
                RelayMode::Open => WIFI_MSG_CHANNEL
                    .sender()
                    .send(MessagePayload::TurnOffNotify),
                RelayMode::Closed => WIFI_MSG_CHANNEL.sender().send(MessagePayload::TurnOnNotify),
            };
        }
    }
}

pub static BUTTON_STATUS: PinStatus<ButtonEvent> = Watch::new();

#[derive(Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ButtonEvent {
    Press,
    Release,
}

/// Pin Level when plug button is pressed
const BUTTON_PRESSED_LEVEL: Level = Level::Low;

async fn button_task(pin: &mut Input<'_>) {
    let sender = BUTTON_STATUS.sender();
    let mut prev_level = pin.level();
    loop {
        pin.wait_for_any_edge().await;
        // debounce de pobre
        Timer::after_millis(50).await;
        let level = pin.level();
        if level == prev_level {
            continue;
        } else if level == BUTTON_PRESSED_LEVEL {
            sender.send(ButtonEvent::Press);
            RELAY_SIGNAL.signal(RELAY_STATUS.try_get().unwrap_or(RelayMode::Closed).toggle());
        } else {
            sender.send(ButtonEvent::Release);
        }
        prev_level = level;
    }
}
