#![warn(rust_2018_idioms)]

//! Integration tests for the UTF-8 wrapper types: `Utf8Buffer`, `Utf8Bytes`,
//! `Utf8BytesMut`. These types guarantee valid UTF-8; the critical test
//! surface is operations involving multi-byte code points and char
//! boundaries.

use smol_bytes::{Utf8Buf, Utf8BufMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut, Utf8Error};

/// A 2-byte char (Latin-1 supplement).
const LATIN_1: &str = "café"; // 'é' = 2 bytes
/// A 3-byte char (BMP).
const BMP: &str = "euro€"; // '€' = 3 bytes
/// A 4-byte char (supplementary plane).
const EMOJI: &str = "rust🦀"; // '🦀' = 4 bytes
/// A mix of 1/2/3/4-byte chars.
const MIXED: &str = "a é € 🦀";

// ============================================================================
// Utf8Buffer — 62-byte inline, panics on capacity overflow
// ============================================================================

#[test]
fn utf8_buffer_from_multibyte_str() {
  let b = Utf8Buffer::from(LATIN_1);
  assert_eq!(b.as_str(), LATIN_1);
  assert_eq!(b.len(), LATIN_1.len());

  let b = Utf8Buffer::from(BMP);
  assert_eq!(b.as_str(), BMP);

  let b = Utf8Buffer::from(EMOJI);
  assert_eq!(b.as_str(), EMOJI);

  let b = Utf8Buffer::from(MIXED);
  assert_eq!(b.as_str(), MIXED);
}

#[test]
fn utf8_buffer_push_multibyte_chars() {
  let mut b = Utf8Buffer::new();
  b.push('a');
  b.push('é');
  b.push('€');
  b.push('🦀');
  assert_eq!(b.as_str(), "aé€🦀");
  assert_eq!(b.len(), 1 + 2 + 3 + 4);
}

#[test]
fn utf8_buffer_split_to_on_char_boundary() {
  let mut b = Utf8Buffer::from("café");
  // "café" = [0x63, 0x61, 0x66, 0xc3, 0xa9]; split at 3 (before 'é') is valid
  let head = b.split_to(3);
  assert_eq!(head.as_str(), "caf");
  assert_eq!(b.as_str(), "é");
}

#[test]
#[should_panic]
fn utf8_buffer_split_to_in_middle_of_multibyte_char_panics() {
  let mut b = Utf8Buffer::from("café");
  // "é" is at bytes [3, 4]; splitting at 4 lands in the middle of 'é'
  let _ = b.split_to(4);
}

#[test]
fn utf8_buffer_try_split_to_mid_char_returns_err() {
  let mut b = Utf8Buffer::from("café");
  let r = b.try_split_to(4);
  assert_eq!(r, Err(Utf8Error::InvalidCharBoundary { at: 4 }));
  // Original buffer is untouched on error.
  assert_eq!(b.as_str(), "café");
}

#[test]
fn utf8_buffer_try_split_off_out_of_bounds() {
  let mut b = Utf8Buffer::from("hi");
  let r = b.try_split_off(99);
  assert!(matches!(r, Err(Utf8Error::OutOfBounds { at: 99, len: 2 })));
}

#[test]
fn utf8_buffer_slice_respects_char_boundaries() {
  let b = Utf8Buffer::from("café euro");
  let s = b.slice(3..5); // "é"
  assert_eq!(s.as_str(), "é");
}

#[test]
#[should_panic]
fn utf8_buffer_slice_mid_char_panics() {
  let b = Utf8Buffer::from("café");
  let _ = b.slice(3..4); // mid-é
}

#[test]
fn utf8_buffer_try_slice_mid_char_returns_err() {
  let b = Utf8Buffer::from("café");
  assert!(b.try_slice(3..4).is_err());
}

#[test]
fn utf8_buffer_try_push_overflow() {
  let mut b = Utf8Buffer::from("a".repeat(60).as_str());
  // 60 'a's; capacity is 62. Pushing a 4-byte '🦀' (needs 4 bytes) overflows.
  assert!(b.try_push('🦀').is_err());
  // Content unchanged.
  assert_eq!(b.as_str(), "a".repeat(60).as_str());
}

#[test]
#[should_panic]
fn utf8_buffer_push_overflow_panics() {
  let mut b = Utf8Buffer::from("a".repeat(60).as_str());
  b.push('🦀');
}

#[test]
fn utf8_buffer_try_from_bytes() {
  let valid: &[u8] = "hello".as_bytes();
  let b = Utf8Buffer::try_from(valid).unwrap();
  assert_eq!(b.as_str(), "hello");

  let invalid_utf8: &[u8] = &[0xff, 0xfe];
  assert!(Utf8Buffer::try_from(invalid_utf8).is_err());

  let too_large: Vec<u8> = "a".repeat(100).into_bytes();
  // Valid UTF-8 but >62 bytes; should fail with TooLarge.
  assert!(Utf8Buffer::try_from(too_large.as_slice()).is_err());
}

#[test]
fn utf8_buffer_roundtrip_via_string() {
  let original = Utf8Buffer::from(MIXED);
  let s: String = String::from(original);
  assert_eq!(s, MIXED);
  let back = Utf8Buffer::try_from_str(&s).unwrap();
  assert_eq!(back.as_str(), MIXED);
}

// ============================================================================
// Utf8Bytes — immutable, inline+heap
// ============================================================================

#[test]
fn utf8_bytes_from_multibyte_str() {
  let b = Utf8Bytes::from(MIXED);
  assert_eq!(b.as_str(), MIXED);
}

#[test]
fn utf8_bytes_inline_heap_boundary_ascii() {
  // 62 ASCII bytes: fits inline.
  let s62: String = "a".repeat(62);
  let b = Utf8Bytes::from(s62.as_str());
  assert!(b.is_inline());
  assert_eq!(b.len(), 62);

  // 63 ASCII bytes: goes heap.
  let s63: String = "a".repeat(63);
  let b = Utf8Bytes::from(s63.as_str());
  assert!(b.is_heap());
  assert_eq!(b.len(), 63);
}

#[test]
fn utf8_bytes_inline_heap_boundary_multibyte() {
  // 30 '€' chars = 90 bytes, heap.
  let s = "€".repeat(30);
  let b = Utf8Bytes::from(s.as_str());
  assert!(b.is_heap());
  assert_eq!(b.as_str(), s);
}

#[test]
fn utf8_bytes_split_to_on_char_boundary() {
  let mut b = Utf8Bytes::from("café €uro");
  let head = b.split_to(5); // "café" = 5 bytes
  assert_eq!(head.as_str(), "café");
  assert_eq!(b.as_str(), " €uro");
}

#[test]
#[should_panic]
fn utf8_bytes_split_off_mid_char_panics() {
  let mut b = Utf8Bytes::from("café");
  let _ = b.split_off(4);
}

#[test]
fn utf8_bytes_try_split_off_mid_char_errors() {
  let mut b = Utf8Bytes::from("café");
  assert!(b.try_split_off(4).is_err());
  assert_eq!(b.as_str(), "café");
}

#[test]
fn utf8_bytes_slice_multibyte() {
  let b = Utf8Bytes::from("hello café world");
  let s = b.slice(6..11); // "café"
  assert_eq!(s.as_str(), "café");
}

#[test]
fn utf8_bytes_slice_mid_char_panics_not_silent() {
  let b = Utf8Bytes::from("café");
  assert!(b.try_slice(0..4).is_err()); // ends mid-char
  assert!(b.try_slice(3..5).is_ok()); // ends on boundary
}

#[test]
fn utf8_bytes_try_from_bytes() {
  let valid: &[u8] = "hello world".as_bytes();
  let b = Utf8Bytes::try_from(valid).unwrap();
  assert_eq!(b.as_str(), "hello world");

  let invalid: &[u8] = &[0xff, 0xfe];
  assert!(Utf8Bytes::try_from(invalid).is_err());
}

#[test]
fn utf8_bytes_from_static() {
  let b = Utf8Bytes::from_static("static string");
  assert_eq!(b.as_str(), "static string");
}

#[test]
fn utf8_bytes_roundtrip_via_string() {
  let original = Utf8Bytes::from(MIXED);
  let s: String = String::from(original);
  let back = Utf8Bytes::from(s.as_str());
  assert_eq!(back.as_str(), MIXED);
}

#[test]
fn utf8_bytes_clone_shares_storage() {
  let b = Utf8Bytes::from("a".repeat(100).as_str()); // heap
  let c = b.clone();
  assert_eq!(b.as_str(), c.as_str());
  assert_eq!(b.len(), 100);
}

// ============================================================================
// Utf8BytesMut — mutable, inline+heap, dynamic growth
// ============================================================================

#[test]
fn utf8_bytes_mut_push_multibyte() {
  let mut b = Utf8BytesMut::new();
  b.push('a');
  b.push('é');
  b.push('€');
  b.push('🦀');
  assert_eq!(b.as_str(), "aé€🦀");
  assert_eq!(b.len(), 10);
}

#[test]
fn utf8_bytes_mut_push_str_grows_past_inline() {
  let mut b = Utf8BytesMut::new();
  // Start small, grow past 62 bytes.
  for _ in 0..30 {
    b.push_str("ab");
  }
  assert_eq!(b.len(), 60);
  assert!(b.is_inline());
  b.push_str("cd"); // 62 bytes — still inline
  assert_eq!(b.len(), 62);
  b.push_str("ef"); // 64 bytes — must go heap
  assert_eq!(b.len(), 64);
  assert!(b.is_heap());
}

#[test]
fn utf8_bytes_mut_split_to_on_char_boundary() {
  let mut b = Utf8BytesMut::from("café euro");
  let head = b.split_to(5); // "café"
  assert_eq!(head.as_str(), "café");
  assert_eq!(b.as_str(), " euro");
}

#[test]
#[should_panic]
fn utf8_bytes_mut_split_off_mid_char_panics() {
  let mut b = Utf8BytesMut::from("café");
  let _ = b.split_off(4);
}

#[test]
fn utf8_bytes_mut_try_split_off_mid_char_errors() {
  let mut b = Utf8BytesMut::from("café");
  assert!(b.try_split_off(4).is_err());
  assert_eq!(b.as_str(), "café");
}

#[test]
fn utf8_bytes_mut_slice_multibyte() {
  let b = Utf8BytesMut::from("hello €uro");
  let s = b.slice(6..9); // "€"
  assert_eq!(s.as_str(), "€");
}

#[test]
fn utf8_bytes_mut_split_and_unsplit_heap() {
  let mut b = Utf8BytesMut::with_capacity(128);
  b.push_str("hello café");
  let tail = b.split_off(6); // " café"? no — 6 bytes = "hello "
  assert_eq!(b.as_str(), "hello ");
  assert_eq!(tail.as_str(), "café");
  assert!(b.unsplit(tail).is_none());
  assert_eq!(b.as_str(), "hello café");
}

#[test]
fn utf8_bytes_mut_try_from_bytes() {
  let valid: &[u8] = "hello world".as_bytes();
  let b = Utf8BytesMut::try_from(valid).unwrap();
  assert_eq!(b.as_str(), "hello world");

  let invalid: &[u8] = &[0xff];
  assert!(Utf8BytesMut::try_from(invalid).is_err());
}

#[test]
fn utf8_bytes_mut_clear() {
  let mut b = Utf8BytesMut::from("café");
  b.clear();
  assert!(b.is_empty());
  assert_eq!(b.as_str(), "");
}

#[test]
fn utf8_bytes_mut_inline_heap_boundary_with_multibyte() {
  // 20 '🦀' chars = 80 bytes → heap.
  let s = "🦀".repeat(20);
  let b = Utf8BytesMut::from(s.as_str());
  assert_eq!(b.len(), 80);
  assert!(b.is_heap());
  assert_eq!(b.as_str(), s);
}

// ============================================================================
// Cross-type conversions
// ============================================================================

#[test]
fn cross_type_utf8_buffer_to_utf8_bytes_via_string() {
  let a = Utf8Buffer::from("café");
  let s: String = a.into();
  let b = Utf8Bytes::from(s.as_str());
  assert_eq!(b.as_str(), "café");
}

#[test]
fn cross_type_utf8_bytes_to_bytes_is_zero_validation() {
  use smol_bytes::Bytes;
  let a = Utf8Bytes::from("hello");
  let b: Bytes = a.into();
  assert_eq!(b.as_ref(), b"hello");
}

#[test]
fn cross_type_utf8_bytes_mut_to_bytes_mut() {
  use smol_bytes::BytesMut;
  let a = Utf8BytesMut::from("hello");
  let b: BytesMut = a.into();
  assert_eq!(b.as_slice(), b"hello");
}

#[test]
fn try_from_buffer_with_invalid_utf8() {
  use smol_bytes::Buffer;
  let buf = Buffer::try_from(&[0xffu8, 0xfe][..]).unwrap();
  assert!(Utf8Buffer::try_from(buf).is_err());
}

// ============================================================================
// Cross-type equality
// ============================================================================

#[test]
fn cross_type_eq_all_pairs() {
  let a = Utf8Buffer::from("café");
  let b = Utf8Bytes::from("café");
  let c = Utf8BytesMut::from("café");

  // Both directions of every pair.
  assert_eq!(a, b);
  assert_eq!(b, a);
  assert_eq!(a, c);
  assert_eq!(c, a);
  assert_eq!(b, c);
  assert_eq!(c, b);

  let d = Utf8Buffer::from("different");
  assert_ne!(d, b);
  assert_ne!(d, c);
}
