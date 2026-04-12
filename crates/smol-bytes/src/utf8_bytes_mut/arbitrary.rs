use std::string::String;

use super::*;

impl<'a> ::arbitrary::Arbitrary<'a> for Utf8BytesMut {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    let s = String::arbitrary(u)?;
    Ok(Self::from(s))
  }
}
