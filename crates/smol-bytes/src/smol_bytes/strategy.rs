use super::*;

pub use conversion_friendly::ConversionFriendly;
pub use inline::Inline;

mod conversion_friendly;
mod inline;

pub trait Strategy {
  fn slice(&self, range: impl RangeBounds<usize>) -> Self;

  fn split_to(&mut self, to: usize) -> Self;

  fn split_off(&mut self, at: usize) -> Self;

  fn truncate(&mut self, len: usize);

  fn advance(&mut self, cnt: usize);

  fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes;

  fn clear(&mut self);
}
