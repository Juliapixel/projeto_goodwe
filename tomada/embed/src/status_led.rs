use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::{NoopMutex, raw::CriticalSectionRawMutex},
    signal::Signal,
};
use embassy_time::Timer;
use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};

use crate::RELAY_STATUS;

pub struct StatusLed<'a> {
    plug_led: NoopMutex<Output<'a>>,
    led: NoopMutex<Output<'a>>,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LedStatusCode {
    Disconnected,
    Connecting,
    Working,
    #[default]
    Idle,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PlugLedMode {
    On,
    Off,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OnboardLedMode {
    On,
    Off,
}

pub static LED_STATUS: Signal<CriticalSectionRawMutex, LedStatusCode> = Signal::new();

impl<'a> StatusLed<'a> {
    pub fn new(onboard_pin: impl OutputPin + 'a, plug_pin: impl OutputPin + 'a) -> Self {
        Self {
            led: NoopMutex::new(Output::new(
                onboard_pin,
                Level::High,
                OutputConfig::default(),
            )),
            plug_led: NoopMutex::new(Output::new(plug_pin, Level::High, OutputConfig::default())),
        }
    }

    async fn short_blink(&self) {
        self.led_on();
        Timer::after_millis(100).await;
        self.led_off();
        Timer::after_millis(100).await;
    }

    async fn long_blink(&self) {
        self.led_on();
        Timer::after_millis(500).await;
        self.led_off();
        Timer::after_millis(500).await;
    }

    fn led_off(&self) {
        self.set_plug_led(PlugLedMode::Off);
        self.set_onboard_led(OnboardLedMode::Off);
    }

    fn led_on(&self) {
        self.set_plug_led(PlugLedMode::On);
        self.set_onboard_led(OnboardLedMode::On);
    }

    fn set_onboard_led(&self, mode: OnboardLedMode) {
        // Safety: lock_mut isn't called re-entrantly
        unsafe {
            self.led.lock_mut(|l| l.set_level(mode.into()));
        }
    }

    fn set_plug_led(&self, mode: PlugLedMode) {
        // Safety: lock_mut isn't called re-entrantly
        unsafe {
            self.plug_led.lock_mut(|l| l.set_level(mode.into()));
        }
    }

    pub async fn run(&self) -> ! {
        let mut rcv = RELAY_STATUS.receiver().unwrap();
        let mut blink = async |code: LedStatusCode| match code {
            LedStatusCode::Disconnected => {
                for _ in 0..3 {
                    self.short_blink().await;
                }
                Timer::after_millis(1000).await;
            }
            LedStatusCode::Connecting => {
                self.short_blink().await;
            }
            LedStatusCode::Idle => {
                self.long_blink().await;
            }
            LedStatusCode::Working => {
                let state = rcv.get().await;
                match state {
                    crate::RelayMode::Open => self.led_off(),
                    crate::RelayMode::Closed => self.led_on(),
                };
                match rcv.changed().await {
                    crate::RelayMode::Open => self.led_off(),
                    crate::RelayMode::Closed => self.led_on(),
                };
            }
        };

        let mut code = LED_STATUS.try_take().unwrap_or_default();

        loop {
            match select(LED_STATUS.wait(), async {
                loop {
                    blink(code).await;
                }
            })
            .await
            {
                Either::First(c) => code = c,
                Either::Second(_) => unreachable!(),
            };
        }
    }
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

impl From<OnboardLedMode> for Level {
    fn from(value: OnboardLedMode) -> Self {
        match value {
            OnboardLedMode::On => Level::Low,
            OnboardLedMode::Off => Level::High,
        }
    }
}

impl From<Level> for OnboardLedMode {
    fn from(value: Level) -> Self {
        match value {
            Level::Low => OnboardLedMode::On,
            Level::High => OnboardLedMode::Off,
        }
    }
}
