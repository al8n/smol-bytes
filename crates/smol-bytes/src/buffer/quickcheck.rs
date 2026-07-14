use super::{Buffer, INLINE_CAP};

use quickcheck::Arbitrary;

impl Arbitrary for Buffer {
  fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    let num = usize::arbitrary(g) % INLINE_CAP;
    let mut buf = Self::new();
    for _ in 0..num {
      buf.put_bytes(u8::arbitrary(g), 1);
    }
    buf
  }
}
