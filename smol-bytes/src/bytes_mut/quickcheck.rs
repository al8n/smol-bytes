use super::BytesMut;

use quickcheck::Arbitrary;
use std::{boxed::Box, vec::Vec};

impl Arbitrary for BytesMut {
  fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    BytesMut::from(<Vec<u8> as Arbitrary>::arbitrary(g).as_slice())
  }

  fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
    Box::new(
      <Vec<u8> as Arbitrary>::shrink(&self.to_vec()).map(|vec| BytesMut::from(vec.as_slice())),
    )
  }
}
