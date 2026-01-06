//! Login packet structures.

use std::net::Ipv4Addr;
use rand::Rng;
use crate::consts::{version, GGNumber, GGStatus};
use crate::hash::gg_login_hash;

/// GG_LOGIN60 packet (0x0015) - sent by client to authenticate.
///
/// After receiving `GG_WELCOME` with the seed, the client computes a password
/// hash using the 32-bit GG hash algorithm and sends this packet.
#[derive(Debug, Clone, PartialEq)]
pub struct GGLogin60 {
  /// Gadu-Gadu user number (UIN).
  pub uin: GGNumber,
  /// Password hash (32-bit, computed with gg_login_hash).
  pub hash: u32,
  /// Initial connection status (see `status` module).
  pub status: GGStatus,
  /// Client version (integer constant, e.g., 0x20 for 6.0).
  pub version: u32,
  /// Unknown field, always 0x00.
  pub unknown1: u8,
  /// Local IP for direct connections.
  pub local_ip: Ipv4Addr,
  /// Local port for direct connections.
  pub local_port: u16,
  /// External IP (usually same as local or 0).
  pub external_ip: Ipv4Addr,
  /// External port.
  pub external_port: u16,
  /// Maximum image size in KB that client can receive.
  pub image_size: u8,
  /// Unknown field, always 0xbe.
  pub unknown2: u8,
  /// Status description text (optional, max 70 chars).
  pub description: Option<String>,
  /// Return time (only present if description is present).
  pub time: Option<u32>,
}

impl Default for GGLogin60 {
  fn default() -> Self {
    Self {
      uin: 0,
      hash: 0,
      status: GGStatus::Avail,
      version: version::GG_VERSION_60,
      unknown1: 0x00,
      local_ip: Ipv4Addr::UNSPECIFIED,
      local_port: 0,
      external_ip: Ipv4Addr::UNSPECIFIED,
      external_port: 0,
      image_size: 255,
      unknown2: 0xbe,
      description: None,
      time: None,
    }
  }
}

impl GGLogin60 {
  /// Generate a random GG number (6-8 digits).
  pub fn random_number() -> GGNumber {
    rand::rng().random_range(100_000..100_000_000)
  }

  /// Create a login packet with the given credentials.
  pub fn login(uin: GGNumber, seed: u32, password: &str) -> Self {
    let hash = gg_login_hash(password, seed);
    Self {
      uin,
      hash,
      ..Default::default()
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_login60_default() {
    let login = GGLogin60::default();
    assert_eq!(login.version, version::GG_VERSION_60);
    assert_eq!(login.unknown1, 0x00);
    assert_eq!(login.unknown2, 0xbe);
    assert_eq!(login.status, GGStatus::Avail);
  }
}
