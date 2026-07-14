use std::string::String;

use super::*;

impl<'a, S> ::arbitrary::Arbitrary<'a> for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    let s = String::arbitrary(u)?;
    Ok(Self::from(s))
  }
}
