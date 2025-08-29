#![cfg_attr(not(feature="std"), no_std)]

#[cfg(all(feature = "std", feature = "no_std"))]
compile_error!("CANNOT HAVE BOTH std AND no_std ENABLED");

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "no_std", derive(defmt::Format))]
pub enum PlugMessage<'a> {
    Ping {
        data: &'a [u8]
    },
    Pong {
        data: &'a [u8]
    },
    TurnOn,
    TurnOff,
    QueryStatus
}
