#![cfg(feature = "serde")]
#![warn(rust_2018_idioms)]

use serde_test::{Token, assert_de_tokens, assert_de_tokens_error, assert_tokens};
use smol_bytes::INLINE_CAP;

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

  let b = Utf8Buffer::try_from("hello").unwrap();
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

  let b = Utf8Buffer::try_from(MIXED).unwrap();
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

  let b = Utf8Buffer::try_from(S).unwrap();
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

#[test]
fn buffer_json_string_enforces_inline_byte_capacity() {
  use smol_bytes::Buffer;

  let at_limit = "a".repeat(62);
  let decoded: Buffer = serde_json::from_str(&serde_json::to_string(&at_limit).unwrap()).unwrap();
  assert_eq!(decoded.as_slice(), at_limit.as_bytes());

  let oversize = "a".repeat(63);
  let error = serde_json::from_str::<Buffer>(&serde_json::to_string(&oversize).unwrap())
    .unwrap_err()
    .to_string();
  assert!(error.contains("requested 63 but only 62 available"));
}

// ---------------------------------------------------------------------------
// `Utf8Bytes` deserialization. The visitor accepts exactly what `String`'s
// visitor accepts, rejects the rest with the same message, and keeps strings
// of at most `INLINE_CAP` bytes inline under both immutable storage strategies.
// ---------------------------------------------------------------------------

const fn ascii(bytes: &'static [u8]) -> &'static str {
  match core::str::from_utf8(bytes) {
    Ok(s) => s,
    Err(_) => panic!("fixture is not UTF-8"),
  }
}

const SHORT: &str = "hello";
const FULL_INLINE: &str = ascii(&[b'a'; INLINE_CAP]);
const MIN_HEAP: &str = ascii(&[b'b'; INLINE_CAP + 1]);
const LONG: &str = ascii(&[b'c'; 130]);
// Twenty 3-byte scalars and two ASCII bytes fill the inline capacity exactly;
// twenty-one 3-byte scalars overshoot it by one byte.
const MULTIBYTE_FULL_INLINE: &str = "€€€€€€€€€€€€€€€€€€€€ab";
const MULTIBYTE_MIN_HEAP: &str = "€€€€€€€€€€€€€€€€€€€€€";
const _: () = {
  assert!(MULTIBYTE_FULL_INLINE.len() == INLINE_CAP);
  assert!(MULTIBYTE_MIN_HEAP.len() == INLINE_CAP + 1);
};

const STRINGS: [&str; 7] = [
  "",
  SHORT,
  FULL_INLINE,
  MIN_HEAP,
  LONG,
  MULTIBYTE_FULL_INLINE,
  MULTIBYTE_MIN_HEAP,
];

/// Hands one value to a visitor through a chosen entry point. `serde_test`
/// drives the same entry points but never returns the value, so storage
/// placement and buffer reuse cannot be observed through it. Only the
/// `deserialize_string` hint, the one `String` uses, is accepted.
enum Input {
  Str(&'static str),
  BorrowedStr(&'static str),
  String(String),
  Bytes(&'static [u8]),
  BorrowedBytes(&'static [u8]),
  ByteBuf(Vec<u8>),
}

impl<'de> serde_core::Deserializer<'de> for Input {
  type Error = serde_core::de::value::Error;

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde_core::de::Visitor<'de>,
  {
    match self {
      Self::Str(v) => visitor.visit_str(v),
      Self::BorrowedStr(v) => visitor.visit_borrowed_str(v),
      Self::String(v) => visitor.visit_string(v),
      Self::Bytes(v) => visitor.visit_bytes(v),
      Self::BorrowedBytes(v) => visitor.visit_borrowed_bytes(v),
      Self::ByteBuf(v) => visitor.visit_byte_buf(v),
    }
  }

  fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
  where
    V: serde_core::de::Visitor<'de>,
  {
    Err(serde_core::de::Error::custom(
      "expected the deserialize_string hint",
    ))
  }

  serde_core::forward_to_deserialize_any! {
    bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
    bytes byte_buf option unit unit_struct newtype_struct seq tuple
    tuple_struct map struct enum identifier ignored_any
  }
}

fn every_input(s: &'static str) -> [Input; 6] {
  [
    Input::Str(s),
    Input::BorrowedStr(s),
    Input::String(s.to_owned()),
    Input::Bytes(s.as_bytes()),
    Input::BorrowedBytes(s.as_bytes()),
    Input::ByteBuf(s.as_bytes().to_vec()),
  ]
}

fn assert_de_string_and_bytes_tokens<T>(s: &'static str)
where
  T: serde_core::de::DeserializeOwned + From<&'static str> + PartialEq + core::fmt::Debug,
{
  let expected = T::from(s);
  let bytes = s.as_bytes();

  assert_de_tokens(&expected, &[Token::Str(s)]);
  assert_de_tokens(&expected, &[Token::BorrowedStr(s)]);
  assert_de_tokens(&expected, &[Token::String(s)]);
  assert_de_tokens(&expected, &[Token::Bytes(bytes)]);
  assert_de_tokens(&expected, &[Token::BorrowedBytes(bytes)]);
  assert_de_tokens(&expected, &[Token::ByteBuf(bytes)]);
}

fn assert_placement_follows_length<T>(s: &'static str, view: fn(&T) -> (&str, bool))
where
  T: serde_core::de::DeserializeOwned,
{
  for input in every_input(s) {
    let value = T::deserialize(input).unwrap();
    let (text, is_inline) = view(&value);
    assert_eq!(text, s);
    assert_eq!(is_inline, s.len() <= INLINE_CAP, "{} bytes", s.len());
  }
}

fn assert_adopts_owned_buffers<T>(as_str: fn(&T) -> &str)
where
  T: serde_core::de::DeserializeOwned,
{
  let owned = String::from(LONG);
  let address = owned.as_ptr();
  let value = T::deserialize(Input::String(owned)).unwrap();
  assert_eq!(as_str(&value).as_ptr(), address, "visit_string copied");

  let owned = LONG.as_bytes().to_vec();
  let address = owned.as_ptr();
  let value = T::deserialize(Input::ByteBuf(owned)).unwrap();
  assert_eq!(as_str(&value).as_ptr(), address, "visit_byte_buf copied");
}

#[test]
fn utf8_bytes_deserializes_every_string_and_bytes_token_like_string() {
  use smol_bytes::{compact, shared};

  for s in STRINGS {
    assert_de_string_and_bytes_tokens::<String>(s);
    assert_de_string_and_bytes_tokens::<shared::Utf8Bytes>(s);
    assert_de_string_and_bytes_tokens::<compact::Utf8Bytes>(s);
  }

  // A `char` reaches `visit_str` through the default forwarding, as for `String`.
  assert_de_tokens(&String::from("é"), &[Token::Char('é')]);
  assert_de_tokens(&shared::Utf8Bytes::from("é"), &[Token::Char('é')]);
  assert_de_tokens(&compact::Utf8Bytes::from("é"), &[Token::Char('é')]);
}

#[test]
fn utf8_bytes_storage_follows_the_inline_capacity_for_every_entry_point() {
  use smol_bytes::{compact, shared};

  for s in STRINGS {
    assert_placement_follows_length::<shared::Utf8Bytes>(s, |v| (v.as_str(), v.is_inline()));
    assert_placement_follows_length::<compact::Utf8Bytes>(s, |v| (v.as_str(), v.is_inline()));
  }
}

#[test]
fn utf8_bytes_adopts_owned_heap_buffers_without_copying() {
  use smol_bytes::{compact, shared};

  assert_adopts_owned_buffers::<shared::Utf8Bytes>(shared::Utf8Bytes::as_str);
  assert_adopts_owned_buffers::<compact::Utf8Bytes>(compact::Utf8Bytes::as_str);
}

#[test]
fn utf8_bytes_rejects_invalid_utf8_with_the_message_string_uses() {
  use smol_bytes::{compact, shared};

  const MESSAGE: &str = "invalid value: byte array, expected a string";
  const LONG_INVALID: [u8; INLINE_CAP + 8] = {
    let mut bytes = [b'a'; INLINE_CAP + 8];
    bytes[INLINE_CAP + 7] = 0xff;
    bytes
  };

  let invalid: [&'static [u8]; 3] = [
    // 0xff never occurs in UTF-8.
    b"ab\xffcd",
    // The first two bytes of a 3-byte scalar.
    b"\xe2\x82",
    // Invalid input longer than the inline capacity.
    &LONG_INVALID,
  ];

  for bytes in invalid {
    for token in [
      Token::Bytes(bytes),
      Token::BorrowedBytes(bytes),
      Token::ByteBuf(bytes),
    ] {
      assert_de_tokens_error::<String>(&[token], MESSAGE);
      assert_de_tokens_error::<shared::Utf8Bytes>(&[token], MESSAGE);
      assert_de_tokens_error::<compact::Utf8Bytes>(&[token], MESSAGE);
    }
  }
}

#[test]
fn utf8_bytes_rejects_non_string_tokens_with_the_message_string_uses() {
  use smol_bytes::{compact, shared};

  let cases: [(&[Token], &str); 5] = [
    (
      &[Token::I32(5)],
      "invalid type: integer `5`, expected a string",
    ),
    (
      &[Token::Bool(true)],
      "invalid type: boolean `true`, expected a string",
    ),
    (
      &[Token::Unit],
      "invalid type: unit value, expected a string",
    ),
    (
      &[Token::None],
      "invalid type: Option value, expected a string",
    ),
    (
      &[Token::Seq { len: Some(0) }, Token::SeqEnd],
      "invalid type: sequence, expected a string",
    ),
  ];

  for (tokens, message) in cases {
    assert_de_tokens_error::<String>(tokens, message);
    assert_de_tokens_error::<shared::Utf8Bytes>(tokens, message);
    assert_de_tokens_error::<compact::Utf8Bytes>(tokens, message);
  }
}

#[test]
fn utf8_bytes_json_unescaped_and_escaped_strings() {
  use smol_bytes::{compact, shared};

  fn assert_decodes<T>(json: &str, expected: &str, view: fn(&T) -> (&str, bool))
  where
    T: serde_core::de::DeserializeOwned,
  {
    let value: T = serde_json::from_str(json).unwrap();
    let (text, is_inline) = view(&value);
    assert_eq!(text, expected);
    assert_eq!(
      is_inline,
      expected.len() <= INLINE_CAP,
      "{} bytes",
      expected.len()
    );
  }

  fn assert_both_decode(json: &str, expected: &str) {
    assert_decodes::<shared::Utf8Bytes>(json, expected, |v| (v.as_str(), v.is_inline()));
    assert_decodes::<compact::Utf8Bytes>(json, expected, |v| (v.as_str(), v.is_inline()));
  }

  // Unescaped: serde_json lends a slice of its input (`visit_borrowed_str`).
  for s in STRINGS {
    let json = serde_json::to_string(s).unwrap();
    assert_eq!(json, format!("\"{s}\""), "fixture needs no escaping");
    assert_both_decode(&json, s);
  }

  // Escaped: serde_json decodes into a scratch buffer and lends that
  // (`visit_str`), so the decoded length, not the JSON length, picks the
  // storage.
  assert_both_decode(
    "\"line\\nbreak \\\"quoted\\\" back\\\\slash\"",
    "line\nbreak \"quoted\" back\\slash",
  );
  assert_both_decode("\"caf\\u00e9 \\ud83e\\udd80\"", "café 🦀");
  for len in [INLINE_CAP, INLINE_CAP + 1] {
    let json = format!("\"{}\"", "\\u0041".repeat(len));
    assert_both_decode(&json, &"A".repeat(len));
  }

  // Serialization is unchanged: the value writes the same JSON as its `str`.
  let special = "tab\t quote\" slash\\ newline\n café 🦀";
  let json = serde_json::to_string(&shared::Utf8Bytes::from(special)).unwrap();
  assert_eq!(json, serde_json::to_string(special).unwrap());
  assert_both_decode(&json, special);
}
