use crate::buffer::Buffer;

use std::vec::Vec;

use super::{BytesMut, INLINE_CAP};
use borsh::io::{Read, Write};
use borsh::{BorshDeserialize, BorshSerialize};

impl BorshSerialize for BytesMut {
  fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
    self.as_slice().serialize(writer)
  }
}

impl BorshDeserialize for BytesMut {
  fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
    let len = u32::deserialize_reader(reader)? as usize;
    if len <= INLINE_CAP {
      let mut buf = [0u8; INLINE_CAP];
      reader.read_exact(&mut buf[..len])?;
      // Safety: len is guaranteed to be less than or equal to INLINE_CAP
      Ok(Self::from_inline(unsafe { Buffer::from_array(buf, len) }))
    } else {
      // Read incrementally so a corrupt/hostile length prefix cannot force a
      // huge allocation before the payload actually arrives.
      const CHUNK: usize = 4096;
      let mut vec = Vec::with_capacity(len.min(CHUNK));
      let mut chunk = [0u8; CHUNK];
      let mut remaining = len;
      while remaining > 0 {
        let n = remaining.min(CHUNK);
        reader.read_exact(&mut chunk[..n])?;
        vec.extend_from_slice(&chunk[..n]);
        remaining -= n;
      }
      Ok(Self::from(vec))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::vec;

  #[test]
  fn hostile_length_prefix_does_not_preallocate() {
    // A ~2 GiB length prefix with no payload must fail on the first read
    // instead of allocating the claimed length up front.
    let err = ::borsh::from_slice::<BytesMut>(&[0xFF, 0xFF, 0xFF, 0x7F]);
    assert!(err.is_err());
  }

  #[test]
  fn large_roundtrip() {
    let value = BytesMut::from(&vec![7u8; 10_000][..]);
    let encoded = ::borsh::to_vec(&value).unwrap();
    let decoded = ::borsh::from_slice::<BytesMut>(&encoded).unwrap();

    assert_eq!(decoded.as_slice(), value.as_slice());
  }
}
