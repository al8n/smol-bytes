#![warn(rust_2018_idioms)]

//! Coverage of the `Debug`/`LowerHex`/`UpperHex`/`Display`/`fmt::Write` impls
//! across the byte and UTF-8 wrappers. Assertions pin the exact rendered text.

use core::fmt::Write as _;

use smol_bytes::{Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut, compact, shared};

// ---------------------------------------------------------------------------
// Buffer: Debug renders the canonical byte-string form.
// ---------------------------------------------------------------------------

#[test]
fn buffer_debug_escapes_special_bytes() {
  // Covers: printable ASCII, \n, \", \0 and \xNN escaping.
  let buf = Buffer::from([0x61, 0x0a, 0x22, 0x00, 0xab]);
  assert_eq!(format!("{:?}", buf), r#"b"a\n\"\0\xab""#);

  // Covers: \t, \r and the backslash escape arm.
  let buf = Buffer::from([0x09, 0x0d, 0x5c]);
  assert_eq!(format!("{:?}", buf), r#"b"\t\r\\""#);
}

#[test]
fn buffer_lower_and_upper_hex() {
  let buf = Buffer::from([0xab, 0xcd]);
  assert_eq!(format!("{:x}", buf), "abcd");
  assert_eq!(format!("{:X}", buf), "ABCD");

  // Empty renders as the empty string in both cases.
  let empty = Buffer::new();
  assert_eq!(format!("{:x}", empty), "");
  assert_eq!(format!("{:X}", empty), "");
}

#[test]
fn buffer_fmt_write_success_and_full_error() {
  // Success path: writing into a Buffer with room.
  let mut buf = Buffer::new();
  write!(buf, "hi{}", 5).unwrap();
  assert_eq!(buf.as_slice(), b"hi5");

  // Writing exactly up to capacity succeeds.
  let mut boundary = Buffer::new();
  let filler = "a".repeat(62);
  write!(boundary, "{}", filler).unwrap();
  assert_eq!(boundary.len(), 62);

  // Error path: a full buffer rejects further writes and stays unchanged.
  let mut full = Buffer::from([0u8; 62]);
  assert!(write!(full, "x").is_err());
  assert_eq!(full.as_slice(), &[0u8; 62]);
}

// ---------------------------------------------------------------------------
// shared/compact Bytes: fmt forwards to inline Buffer or heap bytes::Bytes.
// ---------------------------------------------------------------------------

#[test]
fn shared_bytes_debug_and_hex_inline_and_heap() {
  // Inline (<= 62 bytes) forwards to Buffer's fmt.
  let inline = shared::Bytes::from_static(b"\xab\xcd");
  assert_eq!(format!("{:x}", inline), "abcd");
  assert_eq!(format!("{:X}", inline), "ABCD");
  assert_eq!(
    format!("{:?}", shared::Bytes::from_static(b"ab")),
    r#"b"ab""#
  );

  // Heap (> 62 bytes) forwards to bytes::Bytes' fmt, which uses the same form.
  let heap = shared::Bytes::from(vec![0x61u8; 70]);
  assert!(heap.is_heap());
  assert_eq!(format!("{:?}", heap), format!("b\"{}\"", "a".repeat(70)));

  let heap_hex = shared::Bytes::from(vec![0xabu8; 70]);
  assert_eq!(format!("{:x}", heap_hex), "ab".repeat(70));
  assert_eq!(format!("{:X}", heap_hex), "AB".repeat(70));
}

#[test]
fn compact_bytes_debug_and_hex() {
  let inline = compact::Bytes::from_static(b"\x01\xff");
  assert_eq!(format!("{:x}", inline), "01ff");
  assert_eq!(format!("{:X}", inline), "01FF");

  let heap = compact::Bytes::from(vec![0x62u8; 70]);
  assert!(heap.is_heap());
  assert_eq!(format!("{:?}", heap), format!("b\"{}\"", "b".repeat(70)));
}

// ---------------------------------------------------------------------------
// BytesMut: fmt forwards to inline Buffer or heap bytes::BytesMut.
// ---------------------------------------------------------------------------

#[test]
fn bytes_mut_debug_and_hex_inline_and_heap() {
  let inline = BytesMut::from(&b"\xab\xcd"[..]);
  assert!(inline.is_inline());
  assert_eq!(format!("{:x}", inline), "abcd");
  assert_eq!(format!("{:X}", inline), "ABCD");
  assert_eq!(format!("{:?}", BytesMut::from(&b"ab"[..])), r#"b"ab""#);

  let mut heap = BytesMut::with_capacity(128);
  heap.extend_from_slice(&[0x63u8; 70]);
  assert!(heap.is_heap());
  assert_eq!(format!("{:?}", heap), format!("b\"{}\"", "c".repeat(70)));
  assert_eq!(format!("{:x}", heap), "63".repeat(70));
}

#[test]
fn bytes_mut_fmt_write_appends() {
  let mut buf = BytesMut::new();
  write!(buf, "val={}", 42).unwrap();
  assert_eq!(buf.as_slice(), b"val=42");
}

// ---------------------------------------------------------------------------
// UTF-8 wrappers: Debug is the quoted string, Display is the raw content.
// ---------------------------------------------------------------------------

#[test]
fn utf8_buffer_debug_and_display() {
  let buf = Utf8Buffer::from("a\"b");
  assert_eq!(format!("{:?}", buf), r#""a\"b""#);
  assert_eq!(format!("{}", Utf8Buffer::from("héllo")), "héllo");
}

#[test]
fn utf8_bytes_debug_and_display_inline_and_heap() {
  // shared inline
  assert_eq!(format!("{:?}", Utf8Bytes::from("hi")), r#""hi""#);
  assert_eq!(format!("{}", Utf8Bytes::from("héllo")), "héllo");

  // shared heap (> 62 bytes)
  let long = "z".repeat(80);
  let heap = Utf8Bytes::from(long.as_str());
  assert_eq!(format!("{}", heap), long);
  assert_eq!(format!("{:?}", heap), format!("{:?}", long.as_str()));

  // compact wrapper
  assert_eq!(format!("{}", compact::Utf8Bytes::from("world")), "world");
}

#[test]
fn utf8_bytes_mut_debug_and_display() {
  let s = Utf8BytesMut::from("caf\u{e9}");
  assert_eq!(format!("{}", s), "café");
  assert_eq!(format!("{:?}", s), r#""café""#);

  let long = "y".repeat(80);
  let heap = Utf8BytesMut::from(long.as_str());
  assert_eq!(format!("{}", heap), long);
}
