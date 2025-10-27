use super::{SmolBytes, Strategy, INLINE_CAP};
use borsh::io::{Read, Write};
use borsh::{BorshDeserialize, BorshSerialize};

impl<S> BorshSerialize for SmolBytes<S>
where
  Self: Strategy,
{
  fn serialize<W: Write>(&self, writer: &mut W) -> borsh::io::Result<()> {
    self.as_slice().serialize(writer)
  }
}

impl<S> BorshDeserialize for SmolBytes<S>
where
  Self: Strategy,
{
  fn deserialize_reader<R: Read>(reader: &mut R) -> borsh::io::Result<Self> {
    let len = u32::deserialize_reader(reader)? as usize;
    if len <= INLINE_CAP {
      let mut buf = [0u8; INLINE_CAP];
      reader.read_exact(&mut buf[..len])?;
      Ok(SmolBytes::inline(buf, 0, len))
    } else {
      let mut vec = vec![0; len];
      reader.read_exact(&mut vec)?;
      Ok(SmolBytes::from(vec))
    }
  }
}
