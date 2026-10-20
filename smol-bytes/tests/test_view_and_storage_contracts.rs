use smol_bytes::{Buf, Bytes, BytesMut, INLINE_CAP, Utf8Buffer, Utf8Bytes, Utf8BytesMut, compact};

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

#[test]
fn bytes_mut_try_make_inline_leaves_inline_cursor_unchanged() {
  let mut bytes = BytesMut::from(&b"abcdef"[..]);
  bytes.advance(2);
  let pointer = bytes.as_slice().as_ptr();
  let capacity = bytes.capacity();

  assert!(bytes.try_make_inline());
  assert!(bytes.is_inline());
  assert_eq!(bytes.as_slice(), b"cdef");
  assert_eq!(bytes.as_slice().as_ptr(), pointer);
  assert_eq!(bytes.capacity(), capacity);
}

#[test]
fn bytes_mut_try_make_inline_rejects_large_views_and_compacts_short_views() {
  let mut too_long = BytesMut::from(vec![b'x'; INLINE_CAP + 1]);
  let too_long_pointer = too_long.as_slice().as_ptr();
  let too_long_capacity = too_long.capacity();
  assert!(too_long.is_heap());
  assert!(!too_long.try_make_inline());
  assert!(too_long.is_heap());
  assert_eq!(too_long.as_slice().as_ptr(), too_long_pointer);
  assert_eq!(too_long.capacity(), too_long_capacity);

  let mut short = BytesMut::with_capacity(128);
  short.extend_from_slice(b"short");
  assert!(short.is_heap());
  assert!(short.try_make_inline());
  assert_eq!(short.as_slice(), b"short");
  assert_eq!(short.capacity(), INLINE_CAP);

  let mut empty = BytesMut::with_capacity(128);
  assert!(empty.is_heap());
  assert!(empty.try_make_inline());
  assert!(empty.is_empty());
  assert_eq!(empty.capacity(), INLINE_CAP);

  let mut advanced = BytesMut::with_capacity(128);
  advanced.extend_from_slice(b"prefix-visible");
  advanced.advance(b"prefix-".len());
  assert!(advanced.is_heap());
  assert!(advanced.capacity() > INLINE_CAP);
  assert!(advanced.try_make_inline());
  assert_eq!(advanced.as_slice(), b"visible");
  assert_eq!(advanced.capacity(), INLINE_CAP);

  let payload = vec![b's'; 80];
  let mut split = BytesMut::with_capacity(128);
  split.extend_from_slice(&payload);
  let sibling = split.split_to(18);
  let sibling_pointer = sibling.as_slice().as_ptr();
  let sibling_capacity = sibling.capacity();
  assert!(split.is_heap());
  assert_eq!(split.as_slice(), &payload[18..]);
  assert!(split.try_make_inline());
  assert_eq!(split.as_slice(), &payload[18..]);
  assert_eq!(split.capacity(), INLINE_CAP);
  assert_eq!(sibling.as_slice(), &payload[..18]);
  assert_eq!(sibling.as_slice().as_ptr(), sibling_pointer);
  assert_eq!(sibling.capacity(), sibling_capacity);
}

#[test]
fn utf8_bytes_mut_try_make_inline_forwards_multibyte_heap_views() {
  let mut text = Utf8BytesMut::with_capacity(128);
  text.push_str("prefix-é🦀");
  let sibling = text.split_to("prefix-".len());
  let sibling_pointer = sibling.as_str().as_ptr();
  let sibling_capacity = sibling.capacity();

  assert!(text.is_heap());
  assert!(text.try_make_inline());
  assert!(text.is_inline());
  assert_eq!(text.as_str(), "é🦀");
  assert_eq!(text.capacity(), INLINE_CAP);
  assert_eq!(sibling.as_str(), "prefix-");
  assert_eq!(sibling.as_str().as_ptr(), sibling_pointer);
  assert_eq!(sibling.capacity(), sibling_capacity);
}

#[test]
fn owned_utf8_lossy_matches_standard_for_malformed_sequences() {
  let cases: &[&[u8]] = &[
    &[0x80],
    &[0xe2, 0x82],
    &[0xc0, 0xaf],
    &[0xed, 0xa0, 0x80],
    &[0xf4, 0x90, 0x80, 0x80],
  ];

  for input in cases {
    let expected = String::from_utf8_lossy(input).into_owned();
    assert_eq!(
      Utf8Bytes::from_utf8_lossy(Bytes::from(*input)).as_str(),
      expected.as_str()
    );
    assert_eq!(
      compact::Utf8Bytes::from_utf8_lossy(compact::Bytes::from(*input)).as_str(),
      expected.as_str()
    );
    assert_eq!(
      Utf8BytesMut::from_utf8_lossy(BytesMut::from(*input)).as_str(),
      expected.as_str()
    );
  }
}

#[test]
fn owned_utf8_lossy_valid_fast_path_preserves_owned_views() {
  let shared_empty = Utf8Bytes::from_utf8_lossy(Bytes::new());
  let compact_empty = compact::Utf8Bytes::from_utf8_lossy(compact::Bytes::new());
  let mutable_empty = Utf8BytesMut::from_utf8_lossy(BytesMut::new());
  assert!(shared_empty.is_inline());
  assert!(compact_empty.is_inline());
  assert!(mutable_empty.is_inline());

  let shared_inline = Utf8Bytes::from_utf8_lossy(Bytes::from(&b"inline"[..]));
  let compact_inline = compact::Utf8Bytes::from_utf8_lossy(compact::Bytes::from(&b"inline"[..]));
  let mutable_inline = Utf8BytesMut::from_utf8_lossy(BytesMut::from(&b"inline"[..]));
  assert!(shared_inline.is_inline());
  assert!(compact_inline.is_inline());
  assert!(mutable_inline.is_inline());

  let payload = "a".repeat(80);
  let mut shared_input = BytesMut::with_capacity(128);
  shared_input.extend_from_slice(payload.as_bytes());
  let _shared_head = shared_input.split_to(20);
  let shared_raw = shared_input.freeze_shared();
  let shared_pointer = shared_raw.as_slice().as_ptr();
  let shared = Utf8Bytes::from_utf8_lossy(shared_raw);
  assert!(shared.is_heap());
  assert_eq!(shared.as_str(), &payload[20..]);
  assert_eq!(shared.as_str().as_ptr(), shared_pointer);

  let mut compact_input = BytesMut::with_capacity(128);
  compact_input.extend_from_slice(b"short heap");
  let compact_raw = compact_input.freeze_compact();
  let compact_pointer = compact_raw.as_slice().as_ptr();
  assert!(compact_raw.is_heap());
  let compact = compact::Utf8Bytes::from_utf8_lossy(compact_raw);
  assert!(compact.is_heap());
  assert_eq!(compact.as_str(), "short heap");
  assert_eq!(compact.as_str().as_ptr(), compact_pointer);

  let mut mutable_input = BytesMut::with_capacity(128);
  mutable_input.extend_from_slice(b"prefix-visible");
  let _mutable_head = mutable_input.split_to(b"prefix-".len());
  let mutable_pointer = mutable_input.as_slice().as_ptr();
  let mutable_capacity = mutable_input.capacity();
  let mutable = Utf8BytesMut::from_utf8_lossy(mutable_input);
  assert!(mutable.is_heap());
  assert_eq!(mutable.as_str(), "visible");
  assert_eq!(mutable.as_str().as_ptr(), mutable_pointer);
  assert_eq!(mutable.capacity(), mutable_capacity);
}
