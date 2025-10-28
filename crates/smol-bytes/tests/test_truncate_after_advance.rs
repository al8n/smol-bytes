use bytes::Buf;
use smol_bytes::strategy::{compact, shared};

#[test]
fn test_truncate_after_advance_inline_conversion_friendly() {
  let mut b: shared::SmolBytes = shared::SmolBytes::new_inline(b"hello world");
  assert_eq!(b.as_slice(), b"hello world");

  // Advance by 2, so we're now at "llo world"
  b.advance(2);
  assert_eq!(b.as_slice(), b"llo world");

  // Truncate to 5 bytes
  b.truncate(5);
  assert_eq!(
    b.as_slice(),
    b"llo w",
    "truncate should keep first 5 bytes of logical view"
  );
}

#[test]
fn test_truncate_after_advance_inline_inline_strategy() {
  let mut b: compact::SmolBytes = compact::SmolBytes::new_inline(b"hello world");
  assert_eq!(b.as_slice(), b"hello world");

  // Advance by 2, so we're now at "llo world"
  b.advance(2);
  assert_eq!(b.as_slice(), b"llo world");

  // Truncate to 5 bytes
  b.truncate(5);
  assert_eq!(
    b.as_slice(),
    b"llo w",
    "truncate should keep first 5 bytes of logical view"
  );
}

#[test]
fn test_truncate_after_advance_exact_length_conversion_friendly() {
  let mut b: shared::SmolBytes = shared::SmolBytes::new_inline(b"0123456789");
  b.advance(3);
  assert_eq!(b.as_slice(), b"3456789");

  // Truncate to exact remaining length - should be no-op
  b.truncate(7);
  assert_eq!(b.as_slice(), b"3456789");
}

#[test]
fn test_truncate_after_advance_exact_length_inline_strategy() {
  let mut b: compact::SmolBytes = compact::SmolBytes::new_inline(b"0123456789");
  b.advance(3);
  assert_eq!(b.as_slice(), b"3456789");

  // Truncate to exact remaining length - should be no-op
  b.truncate(7);
  assert_eq!(b.as_slice(), b"3456789");
}

#[test]
fn test_truncate_to_zero_after_advance_conversion_friendly() {
  let mut b: shared::SmolBytes = shared::SmolBytes::new_inline(b"hello");
  b.advance(2);
  assert_eq!(b.as_slice(), b"llo");

  b.truncate(0);
  assert_eq!(b.as_slice(), b"");
  assert!(b.is_empty());
}

#[test]
fn test_truncate_to_zero_after_advance_inline_strategy() {
  let mut b: compact::SmolBytes = compact::SmolBytes::new_inline(b"hello");
  b.advance(2);
  assert_eq!(b.as_slice(), b"llo");

  b.truncate(0);
  assert_eq!(b.as_slice(), b"");
  assert!(b.is_empty());
}

#[test]
fn test_multiple_advance_truncate_conversion_friendly() {
  let mut b: shared::SmolBytes = shared::SmolBytes::new_inline(b"abcdefghij");

  b.advance(2); // "cdefghij"
  assert_eq!(b.as_slice(), b"cdefghij");

  b.truncate(5); // "cdefg"
  assert_eq!(b.as_slice(), b"cdefg");

  b.advance(1); // "defg"
  assert_eq!(b.as_slice(), b"defg");

  b.truncate(2); // "de"
  assert_eq!(b.as_slice(), b"de");
}

#[test]
fn test_multiple_advance_truncate_inline_strategy() {
  let mut b: compact::SmolBytes = compact::SmolBytes::new_inline(b"abcdefghij");

  b.advance(2); // "cdefghij"
  assert_eq!(b.as_slice(), b"cdefghij");

  b.truncate(5); // "cdefg"
  assert_eq!(b.as_slice(), b"cdefg");

  b.advance(1); // "defg"
  assert_eq!(b.as_slice(), b"defg");

  b.truncate(2); // "de"
  assert_eq!(b.as_slice(), b"de");
}
