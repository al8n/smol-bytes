/// A strategy for the [`SmolBytes`] type, friendly for conversion between [`bytes::Bytes`] and [`SmolBytes`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConversionFriendly(());
