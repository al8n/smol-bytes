use super::{ImmutableStorage, RawBytes};

impl<'a, S> arbitrary::Arbitrary<'a> for RawBytes<S>
where
  Self: ImmutableStorage,
{
  fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> Result<Self, arbitrary::Error> {
    <&[u8]>::arbitrary(u).map(Self::from)
  }
}
