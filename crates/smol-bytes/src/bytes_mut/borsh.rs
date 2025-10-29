use crate::buffer::Buffer;

use super::{SmolBytesMut, INLINE_CAP};
use borsh::io::{Read, Write};
use borsh::{BorshDeserialize, BorshSerialize};

impl BorshSerialize for SmolBytesMut {
  fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
    self.as_slice().serialize(writer)
  }
}

impl BorshDeserialize for SmolBytesMut {
  fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
    let len = u32::deserialize_reader(reader)? as usize;
    if len <= INLINE_CAP {
      let mut buf = [0u8; INLINE_CAP];
      reader.read_exact(&mut buf[..len])?;
      // Safety: len is guaranteed to be less than or equal to INLINE_CAP
      Ok(Self::from_inline(unsafe { Buffer::from_array(buf, len) }))
    } else {
      let mut vec = Self::zeroed(len);
      reader.read_exact(&mut vec)?;
      Ok(vec)
    }
  }
}
