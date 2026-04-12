use std::{
  boxed::Box,
  string::{String, ToString},
};

use super::*;

impl<S: 'static> ::quickcheck::Arbitrary for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn arbitrary(g: &mut ::quickcheck::Gen) -> Self {
    let s = String::arbitrary(g);
    Self::from(s)
  }

  fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
    let s = self.as_str().to_string();
    Box::new(s.shrink().map(Self::from))
  }
}
