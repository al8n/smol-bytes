#![warn(rust_2018_idioms)]

//! Integration tests for the `const fn` constructors.
//!
//! Every constructor that is documented as usable in a constant is exercised
//! here through a `const` item, so the compile-time path is what is checked:
//! dropping `const` from any of these signatures fails this file at compile
//! time, not at run time.

use smol_bytes::{
  Buffer, Bytes, BytesMut, INLINE_CAP, Utf8Buffer, Utf8Bytes, Utf8BytesMut, compact, shared,
};

/// Fits the inline capacity, so the inline branch of `from_static` is taken.
const SHORT: &str = "hello";
/// Exceeds the inline capacity, so the heap branch of `from_static` is taken.
const LONG: &str =
  "hello world and more data that exceeds the sixty-two byte inline capacity of this buffer";

const _: () = assert!(SHORT.len() <= INLINE_CAP);
const _: () = assert!(LONG.len() > INLINE_CAP);

// ============================================================================
// Buffer / Utf8Buffer — always available, inline only
// ============================================================================

const EMPTY_BUFFER: Buffer = Buffer::new();
const EMPTY_UTF8_BUFFER: Utf8Buffer = Utf8Buffer::new();
const STATIC_UTF8_BUFFER: Utf8Buffer = Utf8Buffer::from_static(SHORT);

#[test]
fn const_buffer_constructors() {
  assert!(EMPTY_BUFFER.is_empty());
  assert_eq!(EMPTY_BUFFER.capacity(), INLINE_CAP);

  assert!(EMPTY_UTF8_BUFFER.is_empty());
  assert_eq!(STATIC_UTF8_BUFFER.as_str(), SHORT);
}

// ============================================================================
// Bytes — both strategies, both storage branches of `from_static`
// ============================================================================

const EMPTY_BYTES: Bytes = Bytes::new();
const INLINE_BYTES: Bytes = Bytes::new_inline(SHORT.as_bytes());
const SHORT_STATIC_BYTES: Bytes = Bytes::from_static(SHORT.as_bytes());
const LONG_STATIC_BYTES: Bytes = Bytes::from_static(LONG.as_bytes());

const EMPTY_COMPACT_BYTES: compact::Bytes = compact::Bytes::new();
const SHORT_STATIC_COMPACT_BYTES: compact::Bytes = compact::Bytes::from_static(SHORT.as_bytes());
const LONG_STATIC_COMPACT_BYTES: compact::Bytes = compact::Bytes::from_static(LONG.as_bytes());

#[test]
fn const_bytes_constructors() {
  assert!(EMPTY_BYTES.is_empty());
  assert!(EMPTY_BYTES.is_inline());

  assert_eq!(&INLINE_BYTES[..], SHORT.as_bytes());
  assert!(INLINE_BYTES.is_inline());

  assert_eq!(&SHORT_STATIC_BYTES[..], SHORT.as_bytes());
  assert!(SHORT_STATIC_BYTES.is_inline());

  assert_eq!(&LONG_STATIC_BYTES[..], LONG.as_bytes());
  assert!(LONG_STATIC_BYTES.is_heap());
}

#[test]
fn const_compact_bytes_constructors() {
  assert!(EMPTY_COMPACT_BYTES.is_empty());
  assert_eq!(&SHORT_STATIC_COMPACT_BYTES[..], SHORT.as_bytes());
  assert_eq!(&LONG_STATIC_COMPACT_BYTES[..], LONG.as_bytes());
}

// ============================================================================
// Utf8Bytes — the const-ified surface, on both strategies
// ============================================================================

const EMPTY_UTF8_BYTES: Utf8Bytes = Utf8Bytes::new();
const SHORT_STATIC_UTF8_BYTES: Utf8Bytes = Utf8Bytes::from_static(SHORT);
const LONG_STATIC_UTF8_BYTES: Utf8Bytes = Utf8Bytes::from_static(LONG);

const EMPTY_SHARED_UTF8_BYTES: shared::Utf8Bytes = shared::Utf8Bytes::new();
const EMPTY_COMPACT_UTF8_BYTES: compact::Utf8Bytes = compact::Utf8Bytes::new();
const SHORT_STATIC_COMPACT_UTF8_BYTES: compact::Utf8Bytes = compact::Utf8Bytes::from_static(SHORT);
const LONG_STATIC_COMPACT_UTF8_BYTES: compact::Utf8Bytes = compact::Utf8Bytes::from_static(LONG);

#[test]
fn const_utf8_bytes_constructors() {
  assert_eq!(EMPTY_UTF8_BYTES.as_str(), "");
  assert!(EMPTY_UTF8_BYTES.is_empty());

  assert_eq!(SHORT_STATIC_UTF8_BYTES.as_str(), SHORT);
  assert!(SHORT_STATIC_UTF8_BYTES.is_inline());

  assert_eq!(LONG_STATIC_UTF8_BYTES.as_str(), LONG);
  assert!(LONG_STATIC_UTF8_BYTES.is_heap());
}

#[test]
fn const_utf8_bytes_constructors_per_strategy() {
  assert_eq!(EMPTY_SHARED_UTF8_BYTES.as_str(), "");
  assert_eq!(EMPTY_COMPACT_UTF8_BYTES.as_str(), "");
  assert_eq!(SHORT_STATIC_COMPACT_UTF8_BYTES.as_str(), SHORT);
  assert_eq!(LONG_STATIC_COMPACT_UTF8_BYTES.as_str(), LONG);
}

// ============================================================================
// Mutable types — only the empty constructors are const; `with_capacity`
// cannot be, because the heap branch allocates.
// ============================================================================

const EMPTY_BYTES_MUT: BytesMut = BytesMut::new();
const EMPTY_UTF8_BYTES_MUT: Utf8BytesMut = Utf8BytesMut::new();

#[test]
fn const_mut_constructors() {
  let mut buf = EMPTY_BYTES_MUT;
  assert!(buf.is_empty());
  buf.extend_from_slice(SHORT.as_bytes());
  assert_eq!(&buf[..], SHORT.as_bytes());

  let mut buf = EMPTY_UTF8_BYTES_MUT;
  assert!(buf.is_empty());
  buf.push_str(SHORT);
  assert_eq!(buf.as_str(), SHORT);
}
