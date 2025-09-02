use std::{net::Ipv4Addr, time::Duration};

use common::{MessageGenerator, MessagePayload};
use env_logger::Env;
use log::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("INFO"));

    let a = tokio::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 8080)).await?;

    let mut generator = MessageGenerator::new(0);

    let mut buf = vec![];
    loop {
        let (bytes, addr) = a.recv_buf_from(&mut buf).await?;
        info!("Received: {bytes} bytes from {addr}");
        let msg = postcard::from_bytes::<common::PlugMessage>(&buf[0..bytes])?;
        info!("received {msg:?}");
        match msg.payload {
            MessagePayload::Ping { data } => {
                a.send_to(
                    &postcard::to_stdvec(&generator.new_message(MessagePayload::Pong { data }))?,
                    addr,
                )
                .await?;
                tokio::time::sleep(Duration::from_secs(5)).await;
                a.send_to(
                    &postcard::to_stdvec(&generator.new_message(MessagePayload::TurnOff))?,
                    addr,
                )
                .await?;
            }
            MessagePayload::TurnOffAck => info!("received TurnOffAck"),
            MessagePayload::Pong { data } => todo!(),
            _ => (),
        }
        buf.clear();
    }
}
