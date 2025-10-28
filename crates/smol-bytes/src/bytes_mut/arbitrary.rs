use super::SmolBytesMut;

impl<'a> arbitrary::Arbitrary<'a> for SmolBytesMut {
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
    <&[u8]>::arbitrary(u).map(Self::from)
  }
}

