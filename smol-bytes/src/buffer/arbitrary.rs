use arbitrary::Arbitrary;

use super::{Buffer, INLINE_CAP};

impl<'a> Arbitrary<'a> for Buffer {
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
    let num: usize = u.arbitrary()?;
    let mut buf = Buffer::new();
    for _ in 0..(num % INLINE_CAP) {
      buf.put_bytes(u.arbitrary()?, 1);
    }
    Ok(buf)
  }
}
