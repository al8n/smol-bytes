use super::*;

impl<'a> ::arbitrary::Arbitrary<'a> for Utf8Buffer {
  fn arbitrary(u: &mut ::arbitrary::Unstructured<'a>) -> ::arbitrary::Result<Self> {
    let s = <&str as ::arbitrary::Arbitrary>::arbitrary(u)?;
    Self::try_from_str(s).map_err(|_| ::arbitrary::Error::IncorrectFormat)
  }

  fn size_hint(depth: usize) -> (usize, Option<usize>) {
    <&str as ::arbitrary::Arbitrary>::size_hint(depth)
  }
}
