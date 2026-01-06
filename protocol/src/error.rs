//! Protocol error types.

use thiserror::Error;
use crate::packets::GGPacket;

#[derive(Error, Debug)]
pub enum GGError {
  #[error("IO error: {0}")]
  Io(#[from] std::io::Error),
  #[error("Unsupported packet type: {0}")]
  UnsupportedPacketType(u32),
  #[error("Unsupported packet: {0:?}")]
  UnsupportedPacket(GGPacket),
}
