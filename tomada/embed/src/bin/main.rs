#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

#[cfg(feature = "ble")]
use bt_hci::controller::ExternalController;
use defmt::{Format, info, write};
use embassy_executor::Spawner;
use embassy_net::StackResources;
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_wifi::EspWifiController;
#[cfg(feature = "ble")]
use esp_wifi::ble::controller::BleConnector;
use esp_wifi::wifi::WifiDevice;
use goodwe_plug::{App, WifiHandler};
#[cfg(feature = "ble")]
use goodwe_plug::BleHandler;
use static_cell::StaticCell;
#[cfg(feature = "ble")]
use trouble_host::{Address, HostResources, prelude::DefaultPacketPool};
use {esp_backtrace as _, esp_println as _};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

struct MacAddressFmt([u8; 6]);

impl Format for MacAddressFmt {
    fn format(&self, fmt: defmt::Formatter) {
        write!(
            fmt,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    // generator version: 0.5.0

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz);
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 64 * 1024);

    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    info!("Embassy initialized!");

    let mut rng = esp_hal::rng::Trng::new(peripherals.RNG, peripherals.ADC1);
    let timer1 = TimerGroup::new(peripherals.TIMG0);

    static WIFI_INIT: StaticCell<EspWifiController<'static>> = StaticCell::new();

    let wifi_init = WIFI_INIT.init(
        esp_wifi::init(timer1.timer0, rng.rng).expect("Failed to initialize WIFI/BLE controller"),
    );

    let (wifi_controller, interfaces) = esp_wifi::wifi::new(wifi_init, peripherals.WIFI)
        .expect("Failed to initialize WIFI controller");

    info!(
        "own MAC address: {}",
        MacAddressFmt(interfaces.sta.mac_address())
    );

    #[cfg(feature = "ble")]
    let transport = BleConnector::new(wifi_init, peripherals.BT);
    #[cfg(feature = "ble")]
    let ble_controller = ExternalController::<_, 20>::new(transport);

    #[cfg(feature = "ble")]
    let mut hr = HostResources::<DefaultPacketPool, 1, 2>::new();
    #[cfg(feature = "ble")]
    let ble_stack = trouble_host::new(ble_controller, &mut hr)
        .set_random_address(Address::random(interfaces.sta.mac_address()))
        .set_random_generator_seed(&mut rng);

    #[cfg(feature = "ble")]
    let ble_host = ble_stack.build();

    let mut stack_resources = StackResources::new();

    let app = App::new(
        peripherals.GPIO8,
        peripherals.GPIO5,
        peripherals.GPIO6,
        peripherals.GPIO7,
        WifiHandler::new(
            wifi_controller,
            interfaces.sta,
            &mut stack_resources,
            rng.random() as u64 | ((rng.random() as u64) << 32),
        ),
        #[cfg(feature = "ble")]
        BleHandler::new(ble_host),
    );

    app.run().await;
}

#[embassy_executor::task]
async fn wifi_runner(mut runner: embassy_net::Runner<'static, WifiDevice<'static>>) {
    runner.run().await;
}
