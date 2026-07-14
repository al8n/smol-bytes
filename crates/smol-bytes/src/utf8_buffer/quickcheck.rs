use crate::INLINE_CAP;

use std::{
  boxed::Box,
  string::{String, ToString},
};

use super::*;

fn truncate_to_inline_char_boundary(s: &str) -> &str {
  let mut len = s.len().min(INLINE_CAP);
  while len > 0 && !s.is_char_boundary(len) {
    len -= 1;
  }
  &s[..len]
}

impl ::quickcheck::Arbitrary for Utf8Buffer {
  fn arbitrary(g: &mut ::quickcheck::Gen) -> Self {
    let s = String::arbitrary(g);
    Self::try_from_str(truncate_to_inline_char_boundary(&s)).unwrap_or_else(|_| Self::new())
  }

  fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
    let s = self.as_str().to_string();
    Box::new(
      s.shrink()
        .filter_map(|shrunken| Self::try_from_str(shrunken.as_str()).ok()),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn truncation_does_not_split_a_multibyte_scalar_at_inline_capacity() {
    let input = std::format!("{}€", "a".repeat(INLINE_CAP - 1));
    let truncated = truncate_to_inline_char_boundary(&input);

    assert_eq!(truncated, "a".repeat(INLINE_CAP - 1));
    assert_eq!(truncated.len(), INLINE_CAP - 1);
    assert!(Utf8Buffer::try_from_str(truncated).is_ok());
  }
}
