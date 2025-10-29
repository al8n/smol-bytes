use super::{RawBytes, Strategy};

impl<'a, S> arbitrary::Arbitrary<'a> for RawBytes<S>
where
  Self: Strategy,
{
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self, arbitrary::Error> {
    <&[u8]>::arbitrary(u).map(Self::from)
  }
}
