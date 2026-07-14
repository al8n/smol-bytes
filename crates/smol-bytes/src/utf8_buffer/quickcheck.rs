use crate::INLINE_CAP;

use std::{
  boxed::Box,
  string::{String, ToString},
};

use super::*;

impl ::quickcheck::Arbitrary for Utf8Buffer {
  fn arbitrary(g: &mut ::quickcheck::Gen) -> Self {
    let s = String::arbitrary(g);
    // Truncate if too long
    let truncated = if s.len() > INLINE_CAP {
      &s[..INLINE_CAP.min(s.len())]
    } else {
      &s
    };

    // Ensure we're on a character boundary
    let mut len = truncated.len();
    while len > 0 && !truncated.is_char_boundary(len) {
      len -= 1;
    }

    Self::try_from(&truncated[..len]).unwrap_or_else(|_| Self::new())
  }

  fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
    let s = self.as_str().to_string();
    Box::new(
      s.shrink()
        .filter_map(|shrunken| Self::try_from(shrunken.as_str()).ok()),
    )
  }
}
