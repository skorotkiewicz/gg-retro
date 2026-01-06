//! GG password hashing algorithms.

/// Compute 32-bit GG password hash.
///
/// This is the hash algorithm used in GG 6.0 and earlier versions.
/// It takes the password and the seed from the welcome packet.
pub fn gg_login_hash(password: &str, seed: u32) -> u32 {
  let mut x: u32 = 0;
  let mut y: u32 = seed;

  for byte in password.bytes() {
    x = (x & 0xffffff00) | (byte as u32);
    y ^= x;
    y = y.wrapping_add(x);
    x <<= 8;
    y ^= x;
    x <<= 8;
    y = y.wrapping_sub(x);
    x <<= 8;
    y ^= x;

    let z = y & 0x1f;
    y = y.rotate_left(z);
  }

  y
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_hashes_password() {
    // Same password + seed should produce same hash
    assert_eq!(
      gg_login_hash("Hello world", 333),
      gg_login_hash("Hello world", 333)
    );

    // Different seed should produce different hash
    assert_ne!(
      gg_login_hash("Hello world", 333),
      gg_login_hash("Hello world", 334)
    );

    // Different password should produce different hash
    assert_ne!(
      gg_login_hash("Hello", 333),
      gg_login_hash("Hello world!", 333)
    );

    // Snapshot test for consistent hash output
    let hash = gg_login_hash("Random password1", 111);
    insta::assert_debug_snapshot!(hash);
  }
}
