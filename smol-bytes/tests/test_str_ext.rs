use core::fmt;

use smol_bytes::{
  INLINE_CAP, StrExt, ToUtf8Bytes, Utf8Bytes, compact, format_compact_utf8bytes, format_utf8bytes,
};

struct DisplayValue {
  number: u8,
}

impl fmt::Display for DisplayValue {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(formatter, "value={}", self.number)
  }
}

struct DisplayText<'a>(&'a str);

impl fmt::Display for DisplayText<'_> {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(self.0)
  }
}

fn assert_compact_inlines_after_shrinking(shared: Utf8Bytes, compact_value: compact::Utf8Bytes) {
  assert!(shared.is_heap());
  assert!(compact_value.is_heap());

  let shared = shared.slice(..INLINE_CAP);
  let compact_value = compact_value.slice(..INLINE_CAP);

  assert!(shared.is_heap());
  assert!(compact_value.is_inline());
  assert_eq!(shared.as_str(), compact_value.as_str());
}

#[test]
fn format_utf8bytes_formats_literals_interpolation_and_unicode() {
  let literal: Utf8Bytes = format_utf8bytes!("literal");
  assert_eq!(literal.as_str(), "literal");
  assert!(literal.is_inline());
  let compact_literal: compact::Utf8Bytes = format_compact_utf8bytes!("literal");
  assert_eq!(compact_literal.as_str(), "literal");
  assert!(compact_literal.is_inline());

  let name = "Ada";
  let interpolated: Utf8Bytes = format_utf8bytes!("hello, {name}!");
  assert_eq!(interpolated.as_str(), "hello, Ada!");
  let compact_interpolated: compact::Utf8Bytes = format_compact_utf8bytes!("hello, {name}!");
  assert_eq!(compact_interpolated.as_str(), "hello, Ada!");

  let crab = '🦀';
  let unicode: Utf8Bytes = format_utf8bytes!("Rust {crab}");
  assert_eq!(unicode.as_str(), "Rust 🦀");
  let compact_unicode: compact::Utf8Bytes = format_compact_utf8bytes!("Rust {crab}");
  assert_eq!(compact_unicode.as_str(), "Rust 🦀");
}

#[test]
fn format_compact_utf8bytes_inlines_after_shrinking() {
  let source = "x".repeat(INLINE_CAP + 1);
  let shared = format_utf8bytes!("{source}");
  let compact_value = format_compact_utf8bytes!("{source}");

  assert_compact_inlines_after_shrinking(shared, compact_value);
}

#[test]
fn to_utf8bytes_uses_display_and_returns_default_shared_type() {
  let value: Utf8Bytes = DisplayValue { number: 7 }.to_utf8bytes();
  assert_eq!(value.as_str(), "value=7");
  assert!(value.is_inline());
}

#[test]
fn to_compact_utf8bytes_inlines_after_shrinking() {
  let source = "x".repeat(INLINE_CAP + 1);
  let display = DisplayText(&source);
  let shared = display.to_utf8bytes();
  let compact_value = display.to_compact_utf8bytes();

  assert_compact_inlines_after_shrinking(shared, compact_value);
}

#[test]
fn str_ext_unicode_case_conversions_match_str() {
  let source = "İstanbul Straße Σ";
  let lowercase = source.to_lowercase();
  let uppercase = source.to_uppercase();

  assert_eq!(source.to_lowercase_utf8bytes().as_str(), lowercase.as_str());
  assert_eq!(source.to_uppercase_utf8bytes().as_str(), uppercase.as_str());
  assert_eq!(
    source.to_lowercase_compact_utf8bytes().as_str(),
    lowercase.as_str()
  );
  assert_eq!(
    source.to_uppercase_compact_utf8bytes().as_str(),
    uppercase.as_str()
  );
}

#[test]
fn str_ext_ascii_case_conversions_only_change_ascii() {
  let source = "RuSt ÄBC";

  assert_eq!(source.to_ascii_lowercase_utf8bytes().as_str(), "rust Äbc");
  assert_eq!(source.to_ascii_uppercase_utf8bytes().as_str(), "RUST ÄBC");
  assert_eq!(
    source.to_ascii_lowercase_compact_utf8bytes().as_str(),
    "rust Äbc"
  );
  assert_eq!(
    source.to_ascii_uppercase_compact_utf8bytes().as_str(),
    "RUST ÄBC"
  );
}

#[test]
fn compact_str_ext_inlines_after_shrinking() {
  let source = "X".repeat(INLINE_CAP + 1);
  let shared = source.to_ascii_lowercase_utf8bytes();
  let compact_value = source.to_ascii_lowercase_compact_utf8bytes();

  assert_compact_inlines_after_shrinking(shared, compact_value);
}

#[test]
fn str_ext_replace_matches_str_replace() {
  let source = "bananana";
  let expected = source.replace("na", "!");

  assert_eq!(
    source.replace_utf8bytes("na", "!").as_str(),
    expected.as_str()
  );
  assert_eq!("abc".replace_utf8bytes("", "-").as_str(), "-a-b-c-");
  assert_eq!(
    source.replace_compact_utf8bytes("na", "!").as_str(),
    expected.as_str()
  );
  assert_eq!("abc".replace_compact_utf8bytes("", "-").as_str(), "-a-b-c-");
}

#[test]
fn str_ext_replacen_respects_count_and_empty_pattern_semantics() {
  assert_eq!("banana".replacen_utf8bytes("na", "!", 1).as_str(), "ba!na");
  assert_eq!("banana".replacen_utf8bytes("na", "!", 0).as_str(), "banana");
  assert_eq!("abc".replacen_utf8bytes("", "-", 2).as_str(), "-a-bc");
  assert_eq!(
    "banana".replacen_compact_utf8bytes("na", "!", 1).as_str(),
    "ba!na"
  );
  assert_eq!(
    "banana".replacen_compact_utf8bytes("na", "!", 0).as_str(),
    "banana"
  );
  assert_eq!(
    "abc".replacen_compact_utf8bytes("", "-", 2).as_str(),
    "-a-bc"
  );
}
