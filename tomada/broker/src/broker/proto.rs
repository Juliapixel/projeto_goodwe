use std::{fmt::Display, io};

use common::PlugMessage;
use tokio_util::{
    bytes::BufMut,
    codec::{Decoder, Encoder},
};

pub struct BrokerCodec;

#[derive(Debug)]
pub enum CodecError {
    DecodeError(postcard::Error),
    Io(io::Error),
}

impl Display for CodecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(io) => write!(f, "{io}"),
            Self::DecodeError(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for CodecError {}

impl From<io::Error> for CodecError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<postcard::Error> for CodecError {
    fn from(value: postcard::Error) -> Self {
        Self::DecodeError(value)
    }
}

impl Decoder for BrokerCodec {
    type Item = PlugMessage;

    type Error = CodecError;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        let msg = postcard::from_bytes::<common::PlugMessage>(src).unwrap();
        src.clear();
        Ok(Some(msg))
    }
}

impl Encoder<PlugMessage> for BrokerCodec {
    type Error = CodecError;

    fn encode(
        &mut self,
        item: PlugMessage,
        dst: &mut tokio_util::bytes::BytesMut,
    ) -> Result<(), Self::Error> {
        postcard::to_io(&item, dst.writer())?;
        Ok(())
    }
}
