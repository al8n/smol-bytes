use super::*;

impl<'a> ::arbitrary::Arbitrary<'a> for Utf8Bytes {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    let s = String::arbitrary(u)?;
    Ok(Self::from(s))
  }
}
