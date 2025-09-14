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
use embassy_time::{Duration, Timer, WithTimeout};
use esp_hal::gpio::{Input, InputConfig, InputPin, Level, Output, OutputConfig, OutputPin, Pull};

use crate::status_led::StatusLed;

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
        let plug_led = Output::new(
            plug_led_pin,
            PlugLedMode::Off.into(),
            OutputConfig::default(),
        );
        let relay = Output::new(relay_pin, RelayMode::Closed.into(), OutputConfig::default());
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
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PlugLedMode {
    On,
    Off,
}

impl From<PlugLedMode> for Level {
    fn from(value: PlugLedMode) -> Self {
        match value {
            PlugLedMode::On => Level::Low,
            PlugLedMode::Off => Level::High,
        }
    }
}

impl From<Level> for PlugLedMode {
    fn from(value: Level) -> Self {
        match value {
            Level::Low => PlugLedMode::On,
            Level::High => PlugLedMode::Off,
        }
    }
}

pub static PLUG_LED_SIGNAL: PinSignal<PlugLedMode> = Signal::new();
pub static PLUG_LED_STATUS: PinStatus<PlugLedMode> = Watch::new();

async fn led_task(pin: &mut Output<'_>) {
    let orig_level = pin.output_level();
    let should_signal = PLUG_LED_STATUS
        .try_get()
        .is_none_or(|l| Level::from(l) == orig_level);
    let sender = PLUG_LED_STATUS.sender();
    if should_signal {
        sender.send(orig_level.into());
    }
    loop {
        let mode = PLUG_LED_SIGNAL.wait().await;
        if mode != pin.output_level().into() {
            pin.set_level(mode.into());
            sender.send(mode);
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

const BUTTON_PRESSED_LEVEL: Level = Level::Low;

async fn button_task(pin: &mut Input<'_>) {
    let sender = BUTTON_STATUS.sender();
    let mut prev_level = pin.level();
    // TODO: do we have to poll it? do interrupts miss?
    loop {
        if pin
            .wait_for_any_edge()
            .with_timeout(Duration::from_millis(50))
            .await
            .is_ok()
        {
            Timer::after_millis(50).await;
        }
        let level = pin.level();
        if level == prev_level {
            continue;
        } else if level == BUTTON_PRESSED_LEVEL {
            sender.send(ButtonEvent::Press);
            let (rmode, lmode) = match RELAY_STATUS.try_get().unwrap_or(RelayMode::Open) {
                RelayMode::Open => (RelayMode::Closed, PlugLedMode::On),
                RelayMode::Closed => (RelayMode::Open, PlugLedMode::Off),
            };
            RELAY_SIGNAL.signal(rmode);
            PLUG_LED_SIGNAL.signal(lmode);
        } else {
            sender.send(ButtonEvent::Release);
        }
        prev_level = level;
    }
}
