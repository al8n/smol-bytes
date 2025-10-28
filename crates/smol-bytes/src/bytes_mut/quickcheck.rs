use super::SmolBytesMut;

use quickcheck::Arbitrary;
use std::{boxed::Box, vec::Vec};


impl Arbitrary for SmolBytesMut {
  fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    SmolBytesMut::from(<Vec<u8> as Arbitrary>::arbitrary(g).as_slice())
  }

  fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
    Box::new(<Vec<u8> as Arbitrary>::shrink(&self.to_vec())
      .map(|vec| SmolBytesMut::from(vec.as_slice())))
  }
}
