#![warn(rust_2018_idioms)]

use std::string::String;

use smol_bytes::{Utf8Bytes, Utf8BytesMut, compact};

#[test]
fn collect_unicode_chars_into_immutable_values() {
  let chars = ['r', 'u', 's', 't', ' ', '🦀'];

  let shared: Utf8Bytes = chars.iter().copied().collect();
  let compact: compact::Utf8Bytes = chars.iter().copied().collect();

  assert_eq!(shared.as_str(), "rust 🦀");
  assert_eq!(compact.as_str(), "rust 🦀");
  assert!(shared.is_inline());
  assert!(compact.is_inline());
}

#[test]
fn collect_long_values_preserves_strategy_after_shrinking() {
  let chars: Vec<char> = std::iter::repeat_n('🦀', 16).collect();
  let mut shared: Utf8Bytes = chars.iter().copied().collect();
  let mut compact: compact::Utf8Bytes = chars.iter().copied().collect();

  assert!(shared.is_heap());
  assert!(compact.is_heap());

  assert_eq!(shared.split_to(4).as_str(), "🦀");
  assert_eq!(compact.split_to(4).as_str(), "🦀");
  assert_eq!(shared.as_str(), "🦀".repeat(15));
  assert_eq!(compact.as_str(), "🦀".repeat(15));
  assert!(shared.is_heap());
  assert!(compact.is_inline());
}

#[test]
fn collect_all_supported_item_forms() {
  let chars = ['a', 'é'];
  let pieces = ["€", "🦀"];
  let owned = vec![String::from("-owned"), String::from("-value")];

  let from_chars: Utf8BytesMut = chars.into_iter().collect();
  let from_char_refs: Utf8BytesMut = chars.iter().collect();
  let from_strs: Utf8BytesMut = pieces.into_iter().collect();
  let from_strings: Utf8BytesMut = owned.clone().into_iter().collect();
  let from_string_refs: Utf8BytesMut = owned.iter().collect();
  let immutable_from_strs: Utf8Bytes = ["€", "🦀"].into_iter().collect();
  let immutable_from_strings: Utf8Bytes = owned.clone().into_iter().collect();
  let immutable_from_string_refs: Utf8Bytes = owned.iter().collect();

  assert_eq!(from_chars.as_str(), "aé");
  assert_eq!(from_char_refs.as_str(), "aé");
  assert_eq!(from_strs.as_str(), "€🦀");
  assert_eq!(from_strings.as_str(), "-owned-value");
  assert_eq!(from_string_refs.as_str(), "-owned-value");
  assert_eq!(immutable_from_strs.as_str(), "€🦀");
  assert_eq!(immutable_from_strings.as_str(), "-owned-value");
  assert_eq!(immutable_from_string_refs.as_str(), "-owned-value");

  let long: Utf8BytesMut = std::iter::repeat_n('🦀', 16).collect();
  assert!(long.is_heap());
}

#[test]
fn extend_all_supported_item_forms() {
  let chars = ['a', 'é'];
  let pieces = ["€", "🦀"];
  let owned = [String::from("-owned"), String::from("-value")];
  let mut value = Utf8BytesMut::new();

  value.extend(chars);
  value.extend(chars.iter());
  value.extend(pieces.iter().copied());
  value.extend(vec![String::from("-owned"), String::from("-value")]);
  value.extend(owned.iter());

  assert_eq!(value.as_str(), "aéaé€🦀-owned-value-owned-value");
  value.extend(std::iter::repeat_n('🦀', 16));
  assert!(value.is_heap());
}
