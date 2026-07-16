#![cfg(feature = "serde")]
#![warn(rust_2018_idioms)]

use serde_test::{Token, assert_de_tokens, assert_de_tokens_error, assert_tokens};

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
  use smol_bytes::{BytesMut, compact::Bytes};

  let b = Bytes::new();
  assert_tokens(&b, &[Token::Bytes(b"")]);
  let b = BytesMut::with_capacity(0);
  assert_tokens(&b, &[Token::Bytes(b"")]);
}

#[test]
fn test_compact_ser_de() {
  use smol_bytes::{BytesMut, compact::Bytes};

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

// ---------------------------------------------------------------------------
// Visitor coverage for the byte types. serde_test drives each distinct visitor
// method deterministically by token kind; serde_json exercises the array path.
// ---------------------------------------------------------------------------

const OVERSIZE: [u8; 63] = [b'z'; 63];

#[test]
fn buffer_visitor_all_scalar_paths() {
  use smol_bytes::Buffer;

  let expected = Buffer::try_from(&b"hi"[..]).unwrap();

  // Serialize uses serialize_bytes -> Token::Bytes; assert_tokens covers both
  // serialization and the visit_bytes deserialization path.
  assert_tokens(&expected, &[Token::Bytes(b"hi")]);

  // Each remaining visitor method, selected by token kind.
  assert_de_tokens(&expected, &[Token::BorrowedBytes(b"hi")]);
  assert_de_tokens(&expected, &[Token::ByteBuf(b"hi")]);
  assert_de_tokens(&expected, &[Token::Str("hi")]);
  assert_de_tokens(&expected, &[Token::BorrowedStr("hi")]);
  assert_de_tokens(&expected, &[Token::String("hi")]);
}

#[test]
fn buffer_visitor_seq_and_errors() {
  use smol_bytes::Buffer;

  // visit_seq with an in-range size hint accumulates the bytes.
  let expected = Buffer::try_from(&b"hi"[..]).unwrap();
  assert_de_tokens(
    &expected,
    &[
      Token::Seq { len: Some(2) },
      Token::U8(b'h'),
      Token::U8(b'i'),
      Token::SeqEnd,
    ],
  );

  // visit_seq with an unknown size hint still works (the None arm).
  assert_de_tokens(
    &expected,
    &[
      Token::Seq { len: None },
      Token::U8(b'h'),
      Token::U8(b'i'),
      Token::SeqEnd,
    ],
  );

  // A size hint beyond the inline capacity is rejected up front.
  assert_de_tokens_error::<Buffer>(
    &[Token::Seq { len: Some(63) }, Token::SeqEnd],
    "too many bytes",
  );

  // Too many bytes for the inline buffer surfaces the TryPutError message.
  assert_de_tokens_error::<Buffer>(
    &[Token::Bytes(&OVERSIZE)],
    "Not enough bytes remaining in buffer to write value (requested 63 but only 62 available)",
  );
}

#[test]
fn heap_byte_types_visitor_paths() {
  use smol_bytes::{BytesMut, compact, shared};

  // shared::Bytes
  let expected = shared::Bytes::from_static(b"hi");
  assert_tokens(&expected, &[Token::Bytes(b"hi")]);
  assert_de_tokens(&expected, &[Token::BorrowedBytes(b"hi")]);
  assert_de_tokens(&expected, &[Token::ByteBuf(b"hi")]);
  assert_de_tokens(&expected, &[Token::Str("hi")]);
  assert_de_tokens(&expected, &[Token::BorrowedStr("hi")]);
  assert_de_tokens(&expected, &[Token::String("hi")]);
  assert_de_tokens(
    &expected,
    &[
      Token::Seq { len: Some(2) },
      Token::U8(b'h'),
      Token::U8(b'i'),
      Token::SeqEnd,
    ],
  );
  assert_de_tokens(
    &expected,
    &[
      Token::Seq { len: None },
      Token::U8(b'h'),
      Token::U8(b'i'),
      Token::SeqEnd,
    ],
  );

  // compact::Bytes
  let expected = compact::Bytes::from_static(b"hi");
  assert_de_tokens(&expected, &[Token::Bytes(b"hi")]);
  assert_de_tokens(&expected, &[Token::Str("hi")]);
  assert_de_tokens(
    &expected,
    &[
      Token::Seq { len: Some(2) },
      Token::U8(b'h'),
      Token::U8(b'i'),
      Token::SeqEnd,
    ],
  );

  // BytesMut
  let expected = BytesMut::from(&b"hi"[..]);
  assert_tokens(&expected, &[Token::Bytes(b"hi")]);
  assert_de_tokens(&expected, &[Token::ByteBuf(b"hi")]);
  assert_de_tokens(&expected, &[Token::BorrowedStr("hi")]);
  assert_de_tokens(&expected, &[Token::String("hi")]);
  assert_de_tokens(
    &expected,
    &[
      Token::Seq { len: Some(2) },
      Token::U8(b'h'),
      Token::U8(b'i'),
      Token::SeqEnd,
    ],
  );
}

#[test]
fn byte_types_json_roundtrip_via_seq() {
  use smol_bytes::{Buffer, BytesMut, compact, shared};

  // serialize_bytes yields a JSON array; deserializing it drives visit_seq.
  let buffer = Buffer::try_from(&b"hello"[..]).unwrap();
  let j = serde_json::to_string(&buffer).unwrap();
  assert_eq!(j, "[104,101,108,108,111]");
  let back: Buffer = serde_json::from_str(&j).unwrap();
  assert_eq!(back, buffer);

  let sh = shared::Bytes::from(vec![1u8, 2, 3, 4, 5]);
  let back: shared::Bytes = serde_json::from_str(&serde_json::to_string(&sh).unwrap()).unwrap();
  assert_eq!(back, sh);

  let cp = compact::Bytes::from(vec![9u8; 70]);
  let back: compact::Bytes = serde_json::from_str(&serde_json::to_string(&cp).unwrap()).unwrap();
  assert_eq!(back, cp);

  let bm = BytesMut::from(&b"roundtrip"[..]);
  let back: BytesMut = serde_json::from_str(&serde_json::to_string(&bm).unwrap()).unwrap();
  assert_eq!(back, bm);
}

#[test]
fn buffer_json_string_uses_visit_str() {
  use smol_bytes::Buffer;

  // A JSON string deserializes into a Buffer via the visit_str path.
  let back: Buffer = serde_json::from_str("\"hi\"").unwrap();
  assert_eq!(back, Buffer::try_from(&b"hi"[..]).unwrap());
}
