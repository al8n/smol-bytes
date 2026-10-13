use std::string::String;

use super::*;

impl Extend<char> for Utf8BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
    for ch in iter {
      self.push(ch);
    }
  }
}

impl<'a> Extend<&'a char> for Utf8BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = &'a char>>(&mut self, iter: T) {
    self.extend(iter.into_iter().copied());
  }
}

impl<'a> Extend<&'a str> for Utf8BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
    for s in iter {
      self.push_str(s);
    }
  }
}

impl Extend<String> for Utf8BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
    for s in iter {
      self.push_str(&s);
    }
  }
}

impl<'a> Extend<&'a String> for Utf8BytesMut {
  #[cfg_attr(not(coverage), inline(always))]
  fn extend<T: IntoIterator<Item = &'a String>>(&mut self, iter: T) {
    for s in iter {
      self.push_str(s);
    }
  }
}

impl<T> FromIterator<T> for Utf8BytesMut
where
  Utf8BytesMut: Extend<T>,
{
  #[cfg_attr(not(coverage), inline(always))]
  fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
    let mut bytes = Self::new();
    bytes.extend(iter);
    bytes
  }
}
