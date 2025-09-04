use embassy_futures::select::{Either, select};
use embassy_sync::{
    blocking_mutex::{NoopMutex, raw::CriticalSectionRawMutex},
    signal::Signal,
};
use embassy_time::Timer;
use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};

pub struct StatusLed<'a> {
    led: NoopMutex<Output<'a>>,
}

#[derive(Debug, Clone, Copy, Default, defmt::Format)]
pub enum LedStatusCode {
    Disconnected,
    Connecting,
    Pairing,
    Working,
    #[default]
    Idle,
}

pub static LED_STATUS: Signal<CriticalSectionRawMutex, LedStatusCode> = Signal::new();

impl<'a> StatusLed<'a> {
    pub fn new(pin: impl OutputPin + 'a) -> Self {
        Self {
            led: NoopMutex::new(Output::new(pin, Level::High, OutputConfig::default())),
        }
    }

    async fn long_blink(&self) {
        self.set_onboard_led(Level::Low);
        Timer::after_millis(900).await;
        self.set_onboard_led(Level::High);
        Timer::after_millis(100).await;
    }

    async fn short_blink(&self) {
        self.set_onboard_led(Level::Low);
        Timer::after_millis(100).await;
        self.set_onboard_led(Level::High);
        Timer::after_millis(100).await;
    }

    fn set_onboard_led(&self, level: Level) {
        // Safety: lock_mut isn't called re-entrantly
        unsafe {
            self.led.lock_mut(|l| l.set_level(level));
        }
    }

    pub async fn blink_led(&self) -> ! {
        let get_code = async move || LED_STATUS.wait().await;
        let blink = async |code: LedStatusCode| match code {
            LedStatusCode::Disconnected => {
                for _ in 0..3 {
                    self.short_blink().await;
                }
                Timer::after_millis(1000).await;
            }
            LedStatusCode::Connecting => {
                self.short_blink().await;
            }
            LedStatusCode::Pairing => {
                self.set_onboard_led(Level::Low);
                Timer::after_millis(500).await;
                self.set_onboard_led(Level::High);
                Timer::after_millis(500).await;
            }
            LedStatusCode::Idle => {
                self.set_onboard_led(Level::High);
                core::future::pending::<()>().await;
            }
            LedStatusCode::Working => {
                self.short_blink().await;
                Timer::after_millis(2000).await;
            }
        };

        let mut code = LED_STATUS.try_take().unwrap_or_default();

        loop {
            match select(LED_STATUS.wait(), async move {
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
