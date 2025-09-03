//! GRAHHHHHHHHHHHHHHHHHHHHHHH

use defmt::{debug, error, info, panic};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, lazy_lock::LazyLock, mutex::Mutex};
use embassy_time::Duration;
use esp_wifi::ble::controller::BleConnector;
use futures::future::join;
use static_cell::StaticCell;
use trouble_host::prelude::*;

extern crate alloc;

pub type BleHost<'b> = Host<'b, ExternalController<BleConnector<'b>, 20>, DefaultPacketPool>;
pub type BlePeripheral<'b> =
    Peripheral<'b, ExternalController<BleConnector<'b>, 20>, DefaultPacketPool>;
pub type BleRunner<'b> = Runner<'b, ExternalController<BleConnector<'b>, 20>, DefaultPacketPool>;

pub struct BleHandler<'b> {
    peripheral: BlePeripheral<'b>,
    runner: Mutex<NoopRawMutex, BleRunner<'b>>,
}

enum BleStatus {
    Pairing,
    Connected,
    Disconnected,
}

#[gatt_server]
struct Server {
    plug_service: PlugService,
}

#[gatt_service(uuid = "5131aad8-f51e-4870-8d34-e74c0079646f")]
struct PlugService {
    // #[descriptor(uuid = descriptors::NUMBER_OF_DIGITALS, read, value = [1u8])]
    #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, name = "hello", read, value = "Is Plug On")]
    // #[characteristic(uuid = "a2002e9d-a77f-4771-8794-2aeac5ef45b5", write, read)]
    #[characteristic(uuid = characteristic::BOOLEAN, write, read)]
    pub is_on: [u8; 1],
    // #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, read, value = "Wi-Fi connection SSID")]
    // #[characteristic(uuid = characteristic::FIXED_STRING_64, write, read, notify)]
    // pub ssid: HeaplessString<64>,
    // #[descriptor(uuid = descriptors::MEASUREMENT_DESCRIPTION, read, value = "Wi-Fi connection password")]
    // #[characteristic(uuid = characteristic::FIXED_STRING_64, write, notify)]
    // pub password: HeaplessString<64>,
}

impl<'b> BleHandler<'b> {
    pub fn new(ble_host: BleHost<'b>) -> Self {
        Self {
            peripheral: ble_host.peripheral,
            runner: Mutex::new(ble_host.runner),
        }
    }

    fn get_adv_data() -> &'static [u8] {
        static ADV_DATA: LazyLock<&'static [u8]> = embassy_sync::lazy_lock::LazyLock::new(|| {
            static ADV_DATA_BUF: StaticCell<([u8; 64], usize)> = StaticCell::new();
            let (buf, len) = ADV_DATA_BUF.init_with(|| {
                let mut adv_data = [0u8; 64];
                let uuid = "5131aad8-f51e-4870-8d34-e74c0079646f"
                    .parse::<uuid::Uuid>()
                    .unwrap();
                let adv_data_len = AdStructure::encode_slice(
                    &[
                        AdStructure::ServiceUuids128(&[*uuid.as_bytes()]),
                        AdStructure::CompleteLocalName(b"Tomada Goodwe"),
                        AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                    ],
                    &mut adv_data,
                )
                .unwrap();
                (adv_data, adv_data_len)
            });
            &buf[..*len]
        });

        ADV_DATA.get()
    }

    fn setup(&self) {}

    pub async fn run(&mut self) {
        let server = Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
            name: "Tomada Goodwe",
            appearance: &appearance::power_device::PLUG,
        }))
        .unwrap();
        let logic = async {
            loop {
                let conn = loop {
                    info!("Advertising");
                    let adv_params = AdvertisementParameters {
                        interval_max: Duration::from_millis(1000),
                        interval_min: Duration::from_millis(500),
                        ..Default::default()
                    };
                    let adv = self
                        .peripheral
                        .advertise(
                            &adv_params,
                            Advertisement::ConnectableScannableUndirected {
                                adv_data: BleHandler::get_adv_data(),
                                scan_data: &[],
                            },
                        )
                        .await;
                    info!("Connection request received");

                    match adv {
                        Ok(adv) => match adv.accept().await {
                            Ok(conn) => match conn.with_attribute_server(&server) {
                                Ok(conn) => break conn,
                                Err(e) => {
                                    error!("Failed to advertise: {}", defmt::Debug2Format(&e))
                                }
                            },
                            Err(e) => error!("Failed to advertise: {}", defmt::Debug2Format(&e)),
                        },
                        Err(e) => panic!(
                            "Failed to create BLE advertiser: {}",
                            defmt::Debug2Format(&e)
                        ),
                    }
                };
                info!("Connection estabilished(?)");
                handle_gatt(&server, &conn).await;
            }
        };

        join(
            async {
                let mut runner = self.runner.lock().await;
                loop {
                    runner.run().await.unwrap();
                }
            },
            logic,
        )
        .await;
    }
}

async fn handle_gatt<'s>(server: &'s Server<'_>, conn: &GattConnection<'_, 's, DefaultPacketPool>) {
    let is_on = &server.plug_service.is_on;
    let is_on_handle = is_on.handle;
    // let ssid = &server.plug_service.ssid;
    // let ssid_handle = ssid.handle;
    // let password = &server.plug_service.password;
    // let password_handle = password.handle;

    loop {
        match conn.next().await {
            GattConnectionEvent::Disconnected { reason } => {
                info!("BLE disconnected: {}", defmt::Debug2Format(&reason));
                break;
            }
            GattConnectionEvent::Gatt { event } => {
                let reason = match &event {
                    GattEvent::Read(read_event) => {
                        debug!("Read request received");
                        let handle = read_event.handle();
                        if handle == is_on_handle {
                            server.get(is_on);
                            None
                        }
                        // } else if handle == ssid_handle {
                        //     // server.get(ssid);
                        //     read_event.accept().unwrap().send().await;
                        // } else {
                        //     read_event.reject(AttErrorCode::READ_NOT_PERMITTED).unwrap().send().await;
                        // }
                        else {
                            None
                        }
                    }
                    GattEvent::Write(write_event) => {
                        debug!("Write request received");
                        let handle = write_event.handle();
                        // if handle == ssid_handle {
                        //     server.set(ssid, &HeaplessString::from_gatt(write_event.data()).unwrap());
                        //     write_event.accept().unwrap().send().await;
                        // } else if handle == password_handle {
                        //     server.set(password, &HeaplessString::from_gatt(write_event.data()).unwrap());
                        //     write_event.accept().unwrap().send().await;
                        if handle == is_on_handle {
                            server.set(
                                is_on,
                                &<[u8; 1] as trouble_host::prelude::FromGatt>::from_gatt(
                                    write_event.data(),
                                )
                                .unwrap(),
                            );
                            None
                        } else {
                            Some(AttErrorCode::ATTRIBUTE_NOT_FOUND)
                            // write_event.reject(AttErrorCode::ATTRIBUTE_NOT_FOUND).unwrap().send().await;
                        }
                    }
                    GattEvent::Other(other_event) => {
                        debug!("Received other event");
                        None
                    }
                };
                if let Some(reason) = reason {
                    event.reject(reason).unwrap().send().await
                } else {
                    event.accept().unwrap().send().await;
                }
            }
            other => {
                debug!("Received other gatt connection event")
            }
        }
    }
}
