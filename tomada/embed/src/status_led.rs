use core::pin::pin;

use alloc::rc::Rc;
use embassy_sync::{blocking_mutex::{raw::NoopRawMutex, NoopMutex}, signal::Signal};
use embassy_time::Timer;
use esp_hal::gpio::{Level, Output, OutputConfig, OutputPin};
use futures::future::select;

pub struct StatusLed<'a> {
    led: NoopMutex<Output<'a>>,
    signal: Rc<Signal<NoopRawMutex, LedStatusCode>>
}

#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum LedStatusCode {
    Disconnected,
    Connecting,
    Pairing,
    Working,
    Idle
}

impl<'a> StatusLed<'a> {
    pub fn new(pin: impl OutputPin + 'a) -> (Self, Rc<Signal<NoopRawMutex, LedStatusCode>>) {
        let signal = Rc::new(Signal::new());
        (Self {
            led: NoopMutex::new(Output::new(
                pin,
                Level::High,
                OutputConfig::default()
            )),
            signal: signal.clone(),
        },
        signal)
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
        let get_code = async move || { self.signal.wait().await };
        let blink = async |code: LedStatusCode| {
            match code {
                LedStatusCode::Disconnected => {
                    for _ in 0..4 {
                        self.short_blink().await;
                    }
                    Timer::after_millis(1000).await;
                },
                LedStatusCode::Connecting => {
                    self.short_blink().await;
                },
                LedStatusCode::Pairing => {
                    self.set_onboard_led(Level::Low);
                    Timer::after_millis(500).await;
                    self.set_onboard_led(Level::High);
                    Timer::after_millis(500).await;
                },
                LedStatusCode::Idle => {
                    self.set_onboard_led(Level::High);
                    core::future::pending::<()>().await;
                },
                LedStatusCode::Working => {
                    self.short_blink().await;
                    Timer::after_millis(2000).await;
                }
            }
        };

        let mut code =
            self.signal.try_take().unwrap_or(LedStatusCode::Idle);

        loop {
            match select(
                pin!(get_code()),
                pin!(async move { loop {
                    blink(code).await;
                }})
            ).await {
                futures::future::Either::Left(c) => code = c.0,
                futures::future::Either::Right(_) => unreachable!(),
            };
        }
    }
}
