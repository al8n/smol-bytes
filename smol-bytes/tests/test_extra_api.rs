use smol_bytes::{Buf, BytesMut, INLINE_CAP, Utf8Buffer, Utf8Bytes, Utf8BytesMut, compact};

fn consumed_fixed() -> Utf8Buffer {
  let mut value = Utf8Buffer::try_from("abcdef").unwrap();
  let head = value.split_to(2);
  assert_eq!(head.as_str(), "ab");
  value
}

#[test]
fn utf8_slice_ref_handles_multibyte_views_and_independent_results() {
  let shared = Utf8Bytes::from("aé🦀z");
  let accented = shared.slice_ref(&shared.as_str()[1..3]);
  assert_eq!(accented.as_str(), "é");

  let padded = Utf8Bytes::from("  é🦀  ");
  let trimmed = padded.slice_ref(padded.as_str().trim());
  assert_eq!(trimmed.as_str(), "é🦀");

  let independent = {
    let source = Utf8Bytes::from("héllo");
    source.slice_ref(&source.as_str()[1..3])
  };
  assert_eq!(independent.as_str(), "é");

  let unrelated = String::from("é");
  assert!(shared.try_slice_ref(unrelated.as_str()).is_none());
  assert_eq!(Utf8Bytes::new(), shared.slice_ref(&unrelated[0..0]));
}

#[test]
fn utf8_slice_ref_rejects_hidden_bytes_of_shared_heap_view() {
  let original = Utf8Bytes::from(format!("{}é{}", "p".repeat(64), "s".repeat(64)));
  assert!(original.is_heap());
  let original_text = original.as_str();
  let mut visible = original.clone();
  visible.advance(64);
  visible.truncate('é'.len_utf8());
  assert_eq!(visible.as_str(), "é");
  assert_eq!(
    visible.as_str().as_ptr(),
    original_text.as_ptr().wrapping_add(64)
  );
  assert!(visible.try_slice_ref(&original_text[..64]).is_none());
  assert!(visible.try_slice_ref(&original_text[66..]).is_none());
  assert_eq!(visible.slice_ref(&original_text[64..66]).as_str(), "é");
}

#[test]
fn compact_utf8_slice_ref_applies_existing_threshold_behavior() {
  let text = compact::Utf8Bytes::from("x".repeat(128));
  let large = text.slice_ref(&text.as_str()[16..112]);
  let short = text.slice_ref(&text.as_str()[16..32]);

  assert!(large.is_heap());
  assert_eq!(
    large.as_str().as_ptr(),
    text.as_str().as_ptr().wrapping_add(16)
  );
  assert!(short.is_inline());
  assert_eq!(short.as_str(), &text.as_str()[16..32]);
}

#[test]
fn utf8_buffer_try_reclaim_reclaims_after_split_prefix() {
  let mut text = Utf8Buffer::try_from("aé🦀").unwrap();
  let _prefix = text.split_to(1);
  assert_eq!(text.as_str(), "é🦀");
  assert_eq!(text.capacity(), INLINE_CAP - 1);

  let capacity = text.capacity();
  assert!(!text.try_reclaim(usize::MAX));
  assert_eq!(text.as_str(), "é🦀");
  assert_eq!(text.capacity(), capacity);

  assert!(text.try_reclaim(INLINE_CAP - "é🦀".len()));
  assert_eq!(text.capacity(), INLINE_CAP);
  assert_eq!(text.as_str(), "é🦀");
}

#[test]
fn utf8_bytes_mut_try_reclaim_tracks_outstanding_heap_splits() {
  let mut text = Utf8BytesMut::with_capacity(64);
  text.push_str("abc");
  let held = text.split();
  text.push_str("def");
  let outstanding = text.split();

  assert!(!text.try_reclaim(64));
  drop(held);
  drop(outstanding);
  assert!(text.try_reclaim(64));
}

#[test]
fn utf8_buffer_conversions_preserve_inline_views_and_capacity() {
  let empty: Utf8BytesMut = Utf8Buffer::new().into();
  assert!(empty.is_inline());
  assert!(empty.is_empty());
  assert_eq!(empty.capacity(), INLINE_CAP);

  let full = "a".repeat(INLINE_CAP);
  let mutable: Utf8BytesMut = Utf8Buffer::try_from(full.as_str()).unwrap().into();
  let shared: Utf8Bytes = Utf8Buffer::try_from(full.as_str()).unwrap().into();
  let compact: compact::Utf8Bytes = Utf8Buffer::try_from(full.as_str()).unwrap().into();
  assert!(mutable.is_inline());
  assert!(shared.is_inline());
  assert!(compact.is_inline());
  assert_eq!(mutable.as_str(), full);
  assert_eq!(shared.as_str(), full);
  assert_eq!(compact.as_str(), full);

  let multibyte: Utf8Bytes = Utf8Buffer::try_from("é🦀").unwrap().into();
  assert!(multibyte.is_inline());
  assert_eq!(multibyte.as_str(), "é🦀");

  let mutable: Utf8BytesMut = consumed_fixed().into();
  let shared: Utf8Bytes = consumed_fixed().into();
  let compact: compact::Utf8Bytes = consumed_fixed().into();
  assert_eq!(mutable.as_str(), "cdef");
  assert_eq!(mutable.capacity(), 60);
  assert!(mutable.is_inline());
  let shared_roundtrip: Utf8BytesMut = shared.into();
  let compact_roundtrip: Utf8BytesMut = compact.into();
  assert_eq!(shared_roundtrip.as_str(), "cdef");
  assert_eq!(compact_roundtrip.as_str(), "cdef");
  assert_eq!(shared_roundtrip.capacity(), 60);
  assert_eq!(compact_roundtrip.capacity(), 60);
}

#[test]
fn pop_removes_unicode_scalars_and_respects_visible_views() {
  macro_rules! assert_pops {
    ($value:expr) => {{
      let mut value = $value;
      for expected in ['🦀', '€', 'é', 'a'] {
        assert_eq!(value.pop(), Some(expected));
      }
      assert_eq!(value.pop(), None);
    }};
  }

  assert_pops!(Utf8Buffer::try_from("aé€🦀").unwrap());
  assert_pops!(Utf8BytesMut::from("aé€🦀"));
  assert_pops!(Utf8Bytes::from("aé€🦀"));
  assert_pops!(compact::Utf8Bytes::from("aé€🦀"));

  let mut view = Utf8Bytes::from("aé🦀");
  view.advance(1);
  assert_eq!(view.pop(), Some('🦀'));
  assert_eq!(view.as_str(), "é");
}

#[test]
fn pop_preserves_strategy_storage_at_the_inline_threshold() {
  let input = "a".repeat(INLINE_CAP + 1);

  let mut shared = Utf8Bytes::from(input.as_str());
  let mut compact = compact::Utf8Bytes::from(input.as_str());
  let mut mutable = Utf8BytesMut::from(input);
  assert!(shared.is_heap());
  assert!(compact.is_heap());
  assert!(mutable.is_heap());

  assert_eq!(shared.pop(), Some('a'));
  assert_eq!(compact.pop(), Some('a'));
  assert_eq!(mutable.pop(), Some('a'));
  assert!(shared.is_heap());
  assert!(compact.is_inline());
  assert!(mutable.is_heap());
}

#[test]
fn bytes_mut_reclaim_keeps_content_on_success_and_failure() {
  let mut bytes = BytesMut::with_capacity(64);
  bytes.extend_from_slice(b"abc");
  bytes.advance(2);
  assert!(!bytes.try_reclaim(usize::MAX));
  assert_eq!(bytes.as_slice(), b"c");
  assert!(bytes.try_reclaim(63));
  assert_eq!(bytes.as_slice(), b"c");
}
