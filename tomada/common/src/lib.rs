#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "defmt"))]
compile_error!("CANNOT HAVE BOTH std AND defmt ENABLED");

use core::num::Wrapping;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MessagePayload {
    /// Request from plug to initiate connection
    Conn,
    ConnAck,
    Disconnect {
        reason: DisconnectReason,
    },
    Ping {
        data: [u8; 16],
    },
    Pong {
        data: [u8; 16],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PlugMessage {
    /// sequential ID for dropped packet detection
    pub seq: u32,
    pub payload: MessagePayload,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DisconnectReason {
    BadHeartbeat,
    Timeout,
    ProtocolError,
    SequenceError,
    #[default]
    Closed,
}

impl PlugMessage {
    pub fn new(seq: u32, payload: MessagePayload) -> Self {
        Self { seq, payload }
    }
}
