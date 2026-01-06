//! GG protocol implementation.

pub mod consts;
pub mod error;
pub mod hash;
pub mod packets;
mod codec;
mod codec_helpers;

// Re-export commonly used types
pub use codec::GGCodec;
pub use consts::{packet_type, GGStatus, version, GGNumber};
pub use error::GGError;
pub use hash::gg_login_hash;
pub use packets::{GGPacket, GGLogin60, RichTextFormat, RichTextFormats};
