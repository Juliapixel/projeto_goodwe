#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "no_std"))]
compile_error!("CANNOT HAVE BOTH std AND no_std ENABLED");

use core::num::Wrapping;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "no_std", derive(defmt::Format))]
pub enum MessagePayload<'a> {
    /// Request from plug to initiate connection
    Conn,
    ConnAck,
    Ping {
        data: &'a [u8],
    },
    Pong {
        data: &'a [u8],
    },
    /// Request from broker to turn plug on
    TurnOn,
    TurnOnAck,
    /// Request from broker to turn plug off
    TurnOff,
    TurnOffAck,
    /// Request from broker to query plug status
    QueryStatus,
    StatusResp {
        is_on: bool,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "no_std", derive(defmt::Format))]
pub struct PlugMessage<'a> {
    /// sequential ID for dropped packet detection
    pub id: u32,
    #[serde(borrow)]
    pub payload: MessagePayload<'a>,
}

/// Convenience struct to make new messages with sequential IDs
pub struct MessageGenerator {
    id: Wrapping<u32>,
}

impl<'a> PlugMessage<'a> {
    pub fn new(id: u32, payload: MessagePayload<'a>) -> Self {
        Self { id, payload }
    }
}

impl MessageGenerator {
    pub fn new(id: u32) -> Self {
        Self { id: Wrapping(id) }
    }

    pub fn new_message<'a>(&mut self, payload: MessagePayload<'a>) -> PlugMessage<'a> {
        self.id += 1;
        PlugMessage::new(self.id.0, payload)
    }
}
