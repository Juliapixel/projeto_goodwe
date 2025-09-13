#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "std", feature = "defmt"))]
compile_error!("CANNOT HAVE BOTH std AND defmt ENABLED");

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessagePayload {
    /// Request from plug to initiate connection
    Conn {
        id: uuid::Uuid,
    },
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
    TurnOnNotify,
    /// Request from broker to turn plug off
    TurnOff,
    TurnOffAck,
    TurnOffNotify,
    /// Request from broker to query plug status
    QueryStatus,
    StatusResp {
        is_on: bool,
    },
}

#[cfg(feature = "defmt")]
impl defmt::Format for MessagePayload {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            MessagePayload::Conn { id } => {
                defmt::write!(fmt, "Conn {{ id: {} }}", &defmt::Display2Format(&id))
            }
            MessagePayload::ConnAck => defmt::write!(fmt, "ConnAck"),
            MessagePayload::Disconnect { reason } => {
                defmt::write!(fmt, "Disconnect {{ reason: {} }}", reason)
            }
            MessagePayload::Ping { data } => defmt::write!(fmt, "Ping {{ data: {} }}", data),
            MessagePayload::Pong { data } => defmt::write!(fmt, "Pong {{ data: {} }}", data),
            MessagePayload::TurnOn => defmt::write!(fmt, "TurnOn"),
            MessagePayload::TurnOnAck => defmt::write!(fmt, "TurnOnAck"),
            MessagePayload::TurnOnNotify => defmt::write!(fmt, "TurnOnNotify"),
            MessagePayload::TurnOff => defmt::write!(fmt, "TurnOff"),
            MessagePayload::TurnOffAck => defmt::write!(fmt, "TurnOffAck"),
            MessagePayload::TurnOffNotify => defmt::write!(fmt, "TurnOffNotify"),
            MessagePayload::QueryStatus => defmt::write!(fmt, "QueryStatus"),
            MessagePayload::StatusResp { is_on } => {
                defmt::write!(fmt, "StatusResp {{ is_on: {} }}", is_on)
            }
        }
    }
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
