use std::sync::LazyLock;

use clap::Parser;

pub static ARGS: LazyLock<Args> = LazyLock::new(Args::parse);

#[derive(clap::Parser)]
pub struct Args {
    #[arg(long, default_value_t = 8080)]
    pub broker_port: u16,
    #[arg(long, default_value_t = 8081)]
    pub http_port: u16,
}
