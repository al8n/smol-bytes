use arbitrary::Arbitrary;

use super::{Buffer, INLINE_CAP};

impl<'a> Arbitrary<'a> for Buffer {
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
    let num = u.int_in_range(0..=INLINE_CAP)?;
    let mut buf = Buffer::new();
    for _ in 0..num {
      buf.put_bytes(u.arbitrary()?, 1);
    }
    Ok(buf)
  }
}
