#![cfg(feature = "serde")]
#![warn(rust_2018_idioms)]

use serde_test::{assert_tokens, Token};

#[test]
fn test_ser_de_empty() {
  use smol_bytes::{Bytes, BytesMut};

  let b = Bytes::new();
  assert_tokens(&b, &[Token::Bytes(b"")]);
  let b = BytesMut::with_capacity(0);
  assert_tokens(&b, &[Token::Bytes(b"")]);
}

#[test]
fn test_ser_de() {
  use smol_bytes::{Bytes, BytesMut};

  let b = Bytes::from(&b"bytes"[..]);
  assert_tokens(&b, &[Token::Bytes(b"bytes")]);
  let b = BytesMut::from(&b"bytes"[..]);
  assert_tokens(&b, &[Token::Bytes(b"bytes")]);
}

#[test]
fn test_compact_ser_de_empty() {
  use smol_bytes::{compact::Bytes, BytesMut};

  let b = Bytes::new();
  assert_tokens(&b, &[Token::Bytes(b"")]);
  let b = BytesMut::with_capacity(0);
  assert_tokens(&b, &[Token::Bytes(b"")]);
}

#[test]
fn test_compact_ser_de() {
  use smol_bytes::{compact::Bytes, BytesMut};

  let b = Bytes::from(&b"bytes"[..]);
  assert_tokens(&b, &[Token::Bytes(b"bytes")]);
  let b = BytesMut::from(&b"bytes"[..]);
  assert_tokens(&b, &[Token::Bytes(b"bytes")]);
}

// ---------------------------------------------------------------------------
// UTF-8 wrappers — serialize as strings, not byte arrays.
// ---------------------------------------------------------------------------

#[test]
fn test_utf8_ser_de_empty() {
  use smol_bytes::{Utf8Buffer, Utf8Bytes, Utf8BytesMut};

  let b = Utf8Buffer::new();
  assert_tokens(&b, &[Token::Str("")]);
  let b = Utf8Bytes::new();
  assert_tokens(&b, &[Token::Str("")]);
  let b = Utf8BytesMut::new();
  assert_tokens(&b, &[Token::Str("")]);
}

#[test]
fn test_utf8_ser_de_ascii() {
  use smol_bytes::{Utf8Buffer, Utf8Bytes, Utf8BytesMut};

  let b = Utf8Buffer::from("hello");
  assert_tokens(&b, &[Token::Str("hello")]);
  let b = Utf8Bytes::from("hello");
  assert_tokens(&b, &[Token::Str("hello")]);
  let b = Utf8BytesMut::from("hello");
  assert_tokens(&b, &[Token::Str("hello")]);
}

#[test]
fn test_utf8_ser_de_multibyte() {
  use smol_bytes::{Utf8Buffer, Utf8Bytes, Utf8BytesMut};

  const MIXED: &str = "a é € 🦀";

  let b = Utf8Buffer::from(MIXED);
  assert_tokens(&b, &[Token::Str(MIXED)]);
  let b = Utf8Bytes::from(MIXED);
  assert_tokens(&b, &[Token::Str(MIXED)]);
  let b = Utf8BytesMut::from(MIXED);
  assert_tokens(&b, &[Token::Str(MIXED)]);
}

#[test]
fn test_utf8_ser_de_heap_length() {
  use smol_bytes::{Utf8Bytes, Utf8BytesMut};

  // 120 bytes — heap-backed. assert_tokens requires &'static, so use JSON.
  let long: String = "€".repeat(40);
  let b = Utf8Bytes::from(long.as_str());
  let j = serde_json::to_string(&b).unwrap();
  let back: Utf8Bytes = serde_json::from_str(&j).unwrap();
  assert_eq!(back.as_str(), long.as_str());

  let b = Utf8BytesMut::from(long.as_str());
  let j = serde_json::to_string(&b).unwrap();
  let back: Utf8BytesMut = serde_json::from_str(&j).unwrap();
  assert_eq!(back.as_str(), long.as_str());
}

#[test]
fn test_utf8_json_roundtrip() {
  use smol_bytes::{Utf8Buffer, Utf8Bytes, Utf8BytesMut};

  const S: &str = "café 🦀";

  let b = Utf8Buffer::from(S);
  let j = serde_json::to_string(&b).unwrap();
  assert_eq!(j, format!("\"{}\"", S));
  let back: Utf8Buffer = serde_json::from_str(&j).unwrap();
  assert_eq!(back.as_str(), S);

  let b = Utf8Bytes::from(S);
  let j = serde_json::to_string(&b).unwrap();
  let back: Utf8Bytes = serde_json::from_str(&j).unwrap();
  assert_eq!(back.as_str(), S);

  let b = Utf8BytesMut::from(S);
  let j = serde_json::to_string(&b).unwrap();
  let back: Utf8BytesMut = serde_json::from_str(&j).unwrap();
  assert_eq!(back.as_str(), S);
}
