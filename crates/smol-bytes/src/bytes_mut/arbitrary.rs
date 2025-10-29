use super::BytesMut;

impl<'a> arbitrary::Arbitrary<'a> for BytesMut {
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
    <&[u8]>::arbitrary(u).map(Self::from)
  }
}
