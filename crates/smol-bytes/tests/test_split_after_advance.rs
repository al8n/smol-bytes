use bytes::Buf;
use smol_bytes::shared::SmolBytes;

#[test]
fn test_split_to_after_advance_inline() {
  let mut b: SmolBytes = SmolBytes::new_inline(b"hello world");
  assert_eq!(b.as_slice(), b"hello world");

  // Advance by 2, so we're now at "llo world"
  b.advance(2);
  assert_eq!(b.as_slice(), b"llo world");

  // Split the first 3 bytes "llo"
  let split = b.split_to(3);

  // Expected results:
  assert_eq!(
    split.as_slice(),
    b"llo",
    "split_to should return the first 3 bytes"
  );
  assert_eq!(b.as_slice(), b" world", "remaining should be ' world'");
}

#[test]
fn test_split_off_after_advance_inline() {
  let mut b: SmolBytes = SmolBytes::new_inline(b"hello world");
  assert_eq!(b.as_slice(), b"hello world");

  // Advance by 2, so we're now at "llo world"
  b.advance(2);
  assert_eq!(b.as_slice(), b"llo world");

  // Split off everything after position 3
  let split = b.split_off(3);

  // Expected results:
  assert_eq!(
    b.as_slice(),
    b"llo",
    "split_off should leave the first 3 bytes"
  );
  assert_eq!(
    split.as_slice(),
    b" world",
    "split_off should return ' world'"
  );
}

#[test]
fn test_split_to_at_boundary_after_advance() {
  let mut b: SmolBytes = SmolBytes::new_inline(b"0123456789");
  b.advance(5);
  assert_eq!(b.as_slice(), b"56789");

  let split = b.split_to(5);
  assert_eq!(split.as_slice(), b"56789");
  assert_eq!(b.as_slice(), b"");
}

#[test]
fn test_split_off_at_boundary_after_advance() {
  let mut b: SmolBytes = SmolBytes::new_inline(b"0123456789");
  b.advance(5);
  assert_eq!(b.as_slice(), b"56789");

  let split = b.split_off(0);
  assert_eq!(b.as_slice(), b"");
  assert_eq!(split.as_slice(), b"56789");
}

#[test]
fn test_multiple_operations_inline() {
  let mut b: SmolBytes = SmolBytes::new_inline(b"abcdefghij");

  b.advance(2); // "cdefghij"
  assert_eq!(b.as_slice(), b"cdefghij");

  let s1 = b.split_to(3); // s1="cde", b="fghij"
  assert_eq!(s1.as_slice(), b"cde");
  assert_eq!(b.as_slice(), b"fghij");

  let s2 = b.split_off(2); // b="fg", s2="hij"
  assert_eq!(b.as_slice(), b"fg");
  assert_eq!(s2.as_slice(), b"hij");
}
