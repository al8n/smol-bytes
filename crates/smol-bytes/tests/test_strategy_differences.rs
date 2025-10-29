use bytes::Buf;
use smol_bytes::{compact, shared, INLINE_CAP};

#[test]
fn test_inline_strategy_converts_heap_to_inline_on_advance() {
  // Create heap-allocated SmolBytes (larger than INLINE_CAP)
  let data = vec![1u8; INLINE_CAP + 10];
  let mut b: compact::SmolBytes = compact::SmolBytes::from(data);

  assert!(b.is_heap(), "should start as heap");

  // Advance past the point where remaining fits inline
  b.advance(15);

  assert_eq!(b.len(), INLINE_CAP - 5);
  assert!(!b.is_heap(), "Inline strategy should convert heap->inline");
}

#[test]
fn test_conversion_friendly_keeps_heap_on_advance() {
  // Create heap-allocated SmolBytes (larger than INLINE_CAP)
  let data = vec![1u8; INLINE_CAP + 10];
  let mut b: shared::SmolBytes = shared::SmolBytes::from(data);

  assert!(b.is_heap(), "should start as heap");

  // Advance past the point where remaining fits inline
  b.advance(15);

  assert_eq!(b.len(), INLINE_CAP - 5);
  assert!(
    b.is_heap(),
    "ConversionFriendly should keep heap allocation"
  );
}

#[test]
fn test_inline_strategy_converts_heap_to_inline_on_truncate() {
  let data = vec![1u8; INLINE_CAP + 10];
  let mut b: compact::SmolBytes = compact::SmolBytes::from(data);

  assert!(b.is_heap(), "should start as heap");

  // Truncate to size that fits inline
  b.truncate(INLINE_CAP - 5);

  assert_eq!(b.len(), INLINE_CAP - 5);
  assert!(
    !b.is_heap(),
    "Inline strategy should convert heap->inline on truncate"
  );
}

#[test]
fn test_conversion_friendly_keeps_heap_on_truncate() {
  let data = vec![1u8; INLINE_CAP + 10];
  let mut b: shared::SmolBytes = shared::SmolBytes::from(data);

  assert!(b.is_heap(), "should start as heap");

  // Truncate to size that fits inline
  b.truncate(INLINE_CAP - 5);

  assert_eq!(b.len(), INLINE_CAP - 5);
  assert!(
    b.is_heap(),
    "ConversionFriendly should keep heap allocation on truncate"
  );
}

#[test]
fn test_inline_strategy_converts_heap_to_inline_on_split_to() {
  let data = vec![1u8; INLINE_CAP + 10];
  let mut b: compact::SmolBytes = compact::SmolBytes::from(data);

  assert!(b.is_heap());

  // Split off a large portion, leaving something that fits inline
  let _split = b.split_to(15);

  assert_eq!(b.len(), INLINE_CAP - 5);
  assert!(
    !b.is_heap(),
    "Inline strategy should convert remaining to inline"
  );
}

#[test]
fn test_conversion_friendly_keeps_heap_on_split_to() {
  let data = vec![1u8; INLINE_CAP + 10];
  let mut b: shared::SmolBytes = shared::SmolBytes::from(data);

  assert!(b.is_heap());

  // Split off a large portion, leaving something that fits inline
  let _split = b.split_to(15);

  assert_eq!(b.len(), INLINE_CAP - 5);
  assert!(
    b.is_heap(),
    "ConversionFriendly should keep heap allocation"
  );
}

#[test]
fn test_both_strategies_produce_same_logical_results() {
  let data = b"hello world";

  let mut cf: shared::SmolBytes = shared::SmolBytes::copy_from_slice(data);
  let mut inline: compact::SmolBytes = compact::SmolBytes::copy_from_slice(data);

  // Both should have same content
  assert_eq!(cf.as_slice(), inline.as_slice());

  // After advance
  cf.advance(2);
  inline.advance(2);
  assert_eq!(cf.as_slice(), inline.as_slice());

  // After truncate
  cf.truncate(5);
  inline.truncate(5);
  assert_eq!(cf.as_slice(), inline.as_slice());

  // After split_to
  let cf_split = cf.split_to(2);
  let inline_split = inline.split_to(2);
  assert_eq!(cf_split.as_slice(), inline_split.as_slice());
  assert_eq!(cf.as_slice(), inline.as_slice());
}

#[test]
fn test_slice_produces_same_content_both_strategies() {
  let data = b"hello world from rust";

  let cf: shared::SmolBytes = shared::SmolBytes::copy_from_slice(data);
  let inline: compact::SmolBytes = compact::SmolBytes::copy_from_slice(data);

  let cf_slice = cf.slice(6..11);
  let inline_slice = inline.slice(6..11);

  assert_eq!(cf_slice.as_slice(), b"world");
  assert_eq!(inline_slice.as_slice(), b"world");
  assert_eq!(cf_slice.as_slice(), inline_slice.as_slice());
}
