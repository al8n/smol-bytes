use quickcheck::Arbitrary;

use super::{RawBytes, Strategy};

use std::{boxed::Box, vec::Vec};

impl<S> Arbitrary for RawBytes<S>
where
  RawBytes<S>: Strategy + Clone,
  S: 'static,
{
  fn arbitrary(g: &mut quickcheck::Gen) -> Self {
    Self::from(<Vec<u8> as Arbitrary>::arbitrary(g))
  }

  fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
    Box::new(<Vec<u8> as Arbitrary>::shrink(&self.to_vec()).map(Self::from))
  }
}
