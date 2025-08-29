use std::net::Ipv4Addr;

use common::PlugMessage;
use env_logger::Env;
use log::info;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(Env::new().default_filter_or("INFO"));

    let a = tokio::net::UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 8080)).await?;

    let mut buf = vec![];
    loop {
        let (bytes, addr) = a.recv_buf_from(&mut buf).await?;
        info!("Received: {bytes} bytes from {addr}");
        let msg = postcard::from_bytes::<common::PlugMessage>(&buf[0..bytes])?;
        match msg {
            common::PlugMessage::Ping { data } => {
                a.send_to(&postcard::to_stdvec(&PlugMessage::Pong { data })?, addr).await?;
            },
            common::PlugMessage::Pong { data } => todo!(),
            _ => ()
        }
    }

    Ok(())
}
