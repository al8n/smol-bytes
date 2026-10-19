use core::{
  fmt,
  ops::{Bound, RangeBounds},
};
use std::{
  cell::Cell,
  panic::{self, AssertUnwindSafe},
};

use smol_bytes::{
  Buffer, ByteEditError, INLINE_CAP, OutOfBounds, RangeOutOfBounds, TryPutError,
  TryPutIntegerCheckedError, Utf8Buffer, Utf8EditError, Utf8FormatError, Utf8IndexError,
};

#[cfg(any(feature = "std", feature = "alloc"))]
use smol_bytes::{Buf, Bytes, BytesMut, Utf8Bytes, Utf8BytesMut, compact};

struct CountingRange<'a> {
  start: usize,
  end: usize,
  start_calls: &'a Cell<usize>,
  end_calls: &'a Cell<usize>,
}

impl RangeBounds<usize> for CountingRange<'_> {
  fn start_bound(&self) -> Bound<&usize> {
    let calls = self.start_calls.get();
    assert_eq!(calls, 0, "start bound must only be queried once");
    self.start_calls.set(calls + 1);
    Bound::Included(&self.start)
  }

  fn end_bound(&self) -> Bound<&usize> {
    let calls = self.end_calls.get();
    assert_eq!(calls, 0, "end bound must only be queried once");
    self.end_calls.set(calls + 1);
    Bound::Excluded(&self.end)
  }
}

macro_rules! assert_checked_writers {
  ($new:expr) => {{
    const U7_MAX: u64 = (1_u64 << 56) - 1;
    const I7_MIN: i64 = -(1_i64 << 55);
    const I7_MAX: i64 = (1_i64 << 55) - 1;

    let mut unsigned = $new;
    unsigned.try_put_uint_checked(0, 0).unwrap();
    let before = unsigned.as_slice().to_vec();
    assert!(matches!(
      unsigned.try_put_uint_checked(1, 0),
      Err(TryPutIntegerCheckedError::OutOfRange { nbytes: 0, .. })
    ));
    assert_eq!(unsigned.as_slice(), before);
    unsigned.try_put_uint_checked(u8::MAX.into(), 1).unwrap();
    let before = unsigned.as_slice().to_vec();
    assert!(matches!(
      unsigned.try_put_uint_checked(u64::from(u8::MAX) + 1, 1),
      Err(TryPutIntegerCheckedError::OutOfRange { nbytes: 1, .. })
    ));
    assert_eq!(unsigned.as_slice(), before);
    unsigned.try_put_uint_checked(U7_MAX, 7).unwrap();
    assert!(matches!(
      unsigned.try_put_uint_checked(U7_MAX + 1, 7),
      Err(TryPutIntegerCheckedError::OutOfRange { nbytes: 7, .. })
    ));
    unsigned.try_put_uint_checked(u64::MAX, 8).unwrap();
    assert!(matches!(
      unsigned.try_put_uint_checked(u64::MAX, 9),
      Err(TryPutIntegerCheckedError::InvalidLength(_))
    ));

    let mut unsigned_be = $new;
    unsigned_be.try_put_uint_checked(0x0102, 2).unwrap();
    assert_eq!(unsigned_be.as_slice(), [1, 2]);
    let mut unsigned_le = $new;
    unsigned_le.try_put_uint_checked_le(0x0102, 2).unwrap();
    assert_eq!(unsigned_le.as_slice(), [2, 1]);
    assert!(matches!(
      unsigned_le.try_put_uint_checked_le(U7_MAX + 1, 7),
      Err(TryPutIntegerCheckedError::OutOfRange { nbytes: 7, .. })
    ));
    let mut unsigned_ne = $new;
    unsigned_ne.try_put_uint_checked_ne(0x0102, 2).unwrap();
    assert_eq!(
      unsigned_ne.as_slice(),
      if cfg!(target_endian = "big") {
        [1, 2]
      } else {
        [2, 1]
      }
    );
    unsigned_ne.try_put_uint_checked_ne(u64::MAX, 8).unwrap();

    let mut signed = $new;
    signed.try_put_int_checked(0, 0).unwrap();
    let before = signed.as_slice().to_vec();
    assert!(matches!(
      signed.try_put_int_checked(-1, 0),
      Err(TryPutIntegerCheckedError::OutOfRange { nbytes: 0, .. })
    ));
    assert_eq!(signed.as_slice(), before);
    signed.try_put_int_checked(i8::MIN.into(), 1).unwrap();
    signed.try_put_int_checked(i8::MAX.into(), 1).unwrap();
    let before = signed.as_slice().to_vec();
    assert!(matches!(
      signed.try_put_int_checked(i64::from(i8::MIN) - 1, 1),
      Err(TryPutIntegerCheckedError::OutOfRange { nbytes: 1, .. })
    ));
    assert!(matches!(
      signed.try_put_int_checked(i64::from(i8::MAX) + 1, 1),
      Err(TryPutIntegerCheckedError::OutOfRange { nbytes: 1, .. })
    ));
    assert_eq!(signed.as_slice(), before);
    signed.try_put_int_checked(I7_MIN, 7).unwrap();
    signed.try_put_int_checked(I7_MAX, 7).unwrap();
    let before = signed.as_slice().to_vec();
    assert!(matches!(
      signed.try_put_int_checked(I7_MIN - 1, 7),
      Err(TryPutIntegerCheckedError::OutOfRange { nbytes: 7, .. })
    ));
    assert!(matches!(
      signed.try_put_int_checked(I7_MAX + 1, 7),
      Err(TryPutIntegerCheckedError::OutOfRange { nbytes: 7, .. })
    ));
    assert_eq!(signed.as_slice(), before);
    signed.try_put_int_checked(i64::MIN, 8).unwrap();
    signed.try_put_int_checked(i64::MAX, 8).unwrap();
    assert!(matches!(
      signed.try_put_int_checked(i64::MAX, 9),
      Err(TryPutIntegerCheckedError::InvalidLength(_))
    ));

    let mut signed_be = $new;
    signed_be.try_put_int_checked(-2, 2).unwrap();
    assert_eq!(signed_be.as_slice(), [0xff, 0xfe]);
    let mut signed_le = $new;
    signed_le.try_put_int_checked_le(-2, 2).unwrap();
    assert_eq!(signed_le.as_slice(), [0xfe, 0xff]);
    let mut signed_ne = $new;
    signed_ne.try_put_int_checked_ne(-2, 2).unwrap();
    assert_eq!(
      signed_ne.as_slice(),
      if cfg!(target_endian = "big") {
        [0xff, 0xfe]
      } else {
        [0xfe, 0xff]
      }
    );
    assert!(matches!(
      signed_ne.try_put_int_checked_ne(i64::MAX, 9),
      Err(TryPutIntegerCheckedError::InvalidLength(_))
    ));
  }};
}

#[test]
fn fixed_byte_edits_extensions_and_checked_integers_are_atomic() {
  let mut bytes = Buffer::from(*b"abcdef");
  bytes.advance(2);
  bytes.try_replace_range(1..3, b"XY").unwrap();
  assert_eq!(bytes.as_slice(), b"cXYf");
  bytes.try_extend_from_within(1..3).unwrap();
  assert_eq!(bytes.as_slice(), b"cXYfXY");
  bytes.try_extend_from_within(0..0).unwrap();
  assert_eq!(bytes.as_slice(), b"cXYfXY");

  let start_calls = Cell::new(0);
  let end_calls = Cell::new(0);
  let range = CountingRange {
    start: 1,
    end: 3,
    start_calls: &start_calls,
    end_calls: &end_calls,
  };
  bytes.try_replace_range(range, b"!").unwrap();
  assert_eq!(start_calls.get(), 1);
  assert_eq!(end_calls.get(), 1);
  assert_eq!(bytes.as_slice(), b"c!fXY");

  let start_calls = Cell::new(0);
  let end_calls = Cell::new(0);
  let range = CountingRange {
    start: 1,
    end: 3,
    start_calls: &start_calls,
    end_calls: &end_calls,
  };
  bytes.try_extend_from_within(range).unwrap();
  assert_eq!(start_calls.get(), 1);
  assert_eq!(end_calls.get(), 1);
  assert_eq!(bytes.as_slice(), b"c!fXY!f");

  let mut hidden_prefix = Buffer::from([b'a'; INLINE_CAP]);
  hidden_prefix.advance(4);
  let before = hidden_prefix.as_slice().to_vec();
  assert_eq!(
    hidden_prefix.try_extend_from_within(0..1),
    Err(ByteEditError::Capacity(TryPutError {
      requested: 1,
      available: 0,
    }))
  );
  assert_eq!(hidden_prefix.as_slice(), before);
  assert!(hidden_prefix.try_reclaim(1));
  hidden_prefix.try_extend_from_within(0..1).unwrap();
  assert_eq!(hidden_prefix.len(), INLINE_CAP - 3);

  assert_checked_writers!(Buffer::new());

  let mut capacity_priority = Buffer::from([0_u8; INLINE_CAP]);
  assert!(matches!(
    capacity_priority.try_put_uint_checked(0x100, 1),
    Err(TryPutIntegerCheckedError::OutOfRange { .. })
  ));
  assert_eq!(
    capacity_priority.try_put_uint_checked(1, 1),
    Err(TryPutIntegerCheckedError::NotEnoughSpace(TryPutError {
      requested: 1,
      available: 0,
    }))
  );
  assert!(matches!(
    capacity_priority.try_put_int_checked_le(128, 1),
    Err(TryPutIntegerCheckedError::OutOfRange { .. })
  ));
  assert_eq!(
    capacity_priority.try_put_int_checked_ne(-1, 1),
    Err(TryPutIntegerCheckedError::NotEnoughSpace(TryPutError {
      requested: 1,
      available: 0,
    }))
  );
  let mut legacy = Buffer::new();
  legacy.try_put_uint(0x100, 1).unwrap();
  legacy.try_put_int(128, 1).unwrap();
  assert_eq!(legacy.as_slice(), [0, 0x80]);
}

struct PartialWriter;

impl fmt::Display for PartialWriter {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("a")?;
    formatter.write_str("bb")
  }
}

struct SwallowingWriter;

impl fmt::Display for SwallowingWriter {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    let _ = formatter.write_str("ab");
    let _ = formatter.write_str("cde");
    Ok(())
  }
}

struct FailingFormatter;

impl fmt::Display for FailingFormatter {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("fit")?;
    Err(fmt::Error)
  }
}

struct PanickingFormatter;

impl fmt::Display for PanickingFormatter {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str("ok")?;
    panic!("formatter panic");
  }
}

#[test]
fn fixed_utf8_scalar_edits_extensions_and_formatting_are_transactional() {
  let mut scalars = Utf8Buffer::new();
  for ch in ['a', 'é', '€', '🦀'] {
    scalars.insert(scalars.len(), ch);
  }
  assert_eq!(scalars.as_str(), "aé€🦀");
  for ch in ['a', 'é', '€', '🦀'] {
    assert_eq!(scalars.remove(0), ch);
  }
  assert_eq!(
    scalars.try_remove(0),
    Err(Utf8IndexError::IndexOutOfBounds(OutOfBounds::new(0, 0)))
  );

  let mut text = Utf8Buffer::try_from("aé").unwrap();
  text.try_extend_from_within(1..3).unwrap();
  assert_eq!(text.as_str(), "aéé");
  assert_eq!(
    text.try_insert(2, 'x'),
    Err(Utf8EditError::Index(Utf8IndexError::InvalidCharBoundary {
      index: 2,
    }))
  );
  assert_eq!(
    text.try_remove(text.len()),
    Err(Utf8IndexError::IndexOutOfBounds(OutOfBounds::new(
      text.len(),
      text.len(),
    )))
  );

  let start_calls = Cell::new(0);
  let end_calls = Cell::new(0);
  let range = CountingRange {
    start: 1,
    end: 3,
    start_calls: &start_calls,
    end_calls: &end_calls,
  };
  let mut counted = Utf8Buffer::try_from("aé").unwrap();
  counted.try_extend_from_within(range).unwrap();
  assert_eq!(start_calls.get(), 1);
  assert_eq!(end_calls.get(), 1);
  assert_eq!(counted.as_str(), "aéé");

  let full_text = "a".repeat(INLINE_CAP);
  let mut full = Utf8Buffer::try_from(full_text.as_str()).unwrap();
  assert_eq!(
    full.try_insert(0, 'a'),
    Err(Utf8EditError::Capacity(TryPutError {
      requested: 1,
      available: 0,
    }))
  );
  assert_eq!(full.as_str(), full_text);

  let mut hidden_prefix = Utf8Buffer::try_from("a".repeat(INLINE_CAP).as_str()).unwrap();
  let _ = hidden_prefix.split_to(4);
  assert_eq!(
    hidden_prefix.try_extend_from_within(0..1),
    Err(Utf8EditError::Capacity(TryPutError {
      requested: 1,
      available: 0,
    }))
  );
  assert!(hidden_prefix.try_reclaim(1));
  hidden_prefix.try_extend_from_within(0..1).unwrap();

  let mut partial = Utf8Buffer::try_from("a".repeat(60).as_str()).unwrap();
  let partial_before = partial.as_str().to_owned();
  let partial_pointer = partial.as_str().as_ptr();
  let partial_capacity = partial.capacity();
  assert_eq!(
    partial.try_write_fmt(format_args!("{PartialWriter}")),
    Err(Utf8FormatError::Capacity(TryPutError {
      requested: 2,
      available: 1,
    }))
  );
  assert_eq!(partial.as_str(), partial_before);
  assert_eq!(partial.as_str().as_ptr(), partial_pointer);
  assert_eq!(partial.capacity(), partial_capacity);

  let mut swallowed = Utf8Buffer::try_from("a".repeat(61).as_str()).unwrap();
  let swallowed_before = swallowed.as_str().to_owned();
  let swallowed_pointer = swallowed.as_str().as_ptr();
  let swallowed_capacity = swallowed.capacity();
  assert_eq!(
    swallowed.try_write_fmt(format_args!("{SwallowingWriter}")),
    Err(Utf8FormatError::Capacity(TryPutError {
      requested: 2,
      available: 1,
    }))
  );
  assert_eq!(swallowed.as_str(), swallowed_before);
  assert_eq!(swallowed.as_str().as_ptr(), swallowed_pointer);
  assert_eq!(swallowed.capacity(), swallowed_capacity);

  let mut formatter_error = Utf8Buffer::try_from("prefixkeep").unwrap();
  let _ = formatter_error.split_to("prefix".len());
  let formatter_before = formatter_error.as_str().to_owned();
  let formatter_pointer = formatter_error.as_str().as_ptr();
  let formatter_capacity = formatter_error.capacity();
  assert_eq!(
    formatter_error.try_write_fmt(format_args!("{FailingFormatter}")),
    Err(Utf8FormatError::Formatter(fmt::Error))
  );
  assert_eq!(formatter_error.as_str(), formatter_before);
  assert_eq!(formatter_error.as_str().as_ptr(), formatter_pointer);
  assert_eq!(formatter_error.capacity(), formatter_capacity);

  let mut panic_safe = Utf8Buffer::try_from("keep").unwrap();
  let panic_before = panic_safe.as_str().to_owned();
  let panic_pointer = panic_safe.as_str().as_ptr();
  let panic_capacity = panic_safe.capacity();
  assert!(
    panic::catch_unwind(AssertUnwindSafe(|| {
      let _ = panic_safe.try_write_fmt(format_args!("{PanickingFormatter}"));
    }))
    .is_err()
  );
  assert_eq!(panic_safe.as_str(), panic_before);
  assert_eq!(panic_safe.as_str().as_ptr(), panic_pointer);
  assert_eq!(panic_safe.capacity(), panic_capacity);

  let mut cursor = Utf8Buffer::try_from("abcdef").unwrap();
  let _ = cursor.split_to(2);
  cursor.try_write_fmt(format_args!("!")).unwrap();
  assert_eq!(cursor.as_str(), "cdef!");
  assert_eq!(cursor.capacity(), INLINE_CAP - 2);
}

#[cfg(any(feature = "std", feature = "alloc"))]
#[test]
fn growable_edits_and_reverse_conversions_preserve_visible_views() {
  assert_checked_writers!(BytesMut::new());

  let mut raw = BytesMut::from(&[b'a'; INLINE_CAP][..]);
  raw.try_extend_from_within(..).unwrap();
  assert!(raw.is_heap());
  assert_eq!(raw.len(), INLINE_CAP * 2);

  let start_calls = Cell::new(0);
  let end_calls = Cell::new(0);
  let range = CountingRange {
    start: 1,
    end: 3,
    start_calls: &start_calls,
    end_calls: &end_calls,
  };
  let mut counted_raw = BytesMut::from(&b"abc"[..]);
  counted_raw.try_extend_from_within(range).unwrap();
  assert_eq!(start_calls.get(), 1);
  assert_eq!(end_calls.get(), 1);
  assert_eq!(counted_raw.as_slice(), b"abcbc");

  let mut byte_edit = BytesMut::from(&b"abcdef"[..]);
  byte_edit.try_replace_range(1..3, b"XY").unwrap();
  assert_eq!(byte_edit.as_slice(), b"aXYdef");
  byte_edit.try_replace_range(1..3, b"WXYZ").unwrap();
  assert_eq!(byte_edit.as_slice(), b"aWXYZdef");
  byte_edit.try_replace_range(1..5, b"Q").unwrap();
  assert_eq!(byte_edit.as_slice(), b"aQdef");
  let before = byte_edit.as_slice().to_vec();
  let (start, end) = (4, 2);
  assert_eq!(
    byte_edit.try_replace_range(start..end, b"!"),
    Err(RangeOutOfBounds::new(4, 2, before.len()))
  );
  assert_eq!(byte_edit.as_slice(), before);

  let mut raw_growth = BytesMut::from(&[b'a'; INLINE_CAP - 2][..]);
  raw_growth.try_replace_range(0..1, b"abcde").unwrap();
  assert!(raw_growth.is_heap());

  let mut inline = BytesMut::from(&b"abcdef"[..]);
  inline.advance(2);
  let fixed = inline.try_into_buffer().unwrap();
  assert_eq!(fixed.as_slice(), b"cdef");
  assert_eq!(fixed.capacity(), INLINE_CAP - 2);

  let mut short_heap = BytesMut::with_capacity(128);
  short_heap.extend_from_slice(b"prefix-visible");
  let sibling = short_heap.split_to(b"prefix-".len());
  let fixed = short_heap.try_into_buffer().unwrap();
  assert_eq!(fixed.as_slice(), b"visible");
  assert_eq!(fixed.capacity(), INLINE_CAP);
  assert_eq!(sibling.as_slice(), b"prefix-");

  let mut oversized = BytesMut::with_capacity(128);
  oversized.extend_from_slice(&[b'x'; INLINE_CAP + 1]);
  let pointer = oversized.as_slice().as_ptr();
  let capacity = oversized.capacity();
  let oversized = oversized.try_into_buffer().unwrap_err();
  assert_eq!(oversized.as_slice().as_ptr(), pointer);
  assert_eq!(oversized.capacity(), capacity);

  let shared_oversized = Bytes::from("s".repeat(INLINE_CAP + 1));
  let pointer = shared_oversized.as_slice().as_ptr();
  let shared_oversized = shared_oversized.try_into_buffer().unwrap_err();
  assert!(shared_oversized.is_heap());
  assert_eq!(shared_oversized.as_slice().as_ptr(), pointer);

  let compact_oversized = compact::Bytes::from("c".repeat(INLINE_CAP + 1));
  let pointer = compact_oversized.as_slice().as_ptr();
  let compact_oversized = compact_oversized.try_into_buffer().unwrap_err();
  assert!(compact_oversized.is_heap());
  assert_eq!(compact_oversized.as_slice().as_ptr(), pointer);

  let mut compact_source = BytesMut::with_capacity(128);
  compact_source.extend_from_slice(b"short");
  let compact_short = compact_source.freeze_compact();
  assert!(compact_short.is_heap());
  let fixed = compact_short.try_into_buffer().unwrap();
  assert_eq!(fixed.as_slice(), b"short");
  assert_eq!(fixed.capacity(), INLINE_CAP);

  let mut immutable = Bytes::from("a".repeat(80));
  immutable.advance(75);
  let fixed = immutable.try_into_buffer().unwrap();
  assert_eq!(fixed.as_slice(), b"aaaaa");
  assert_eq!(fixed.capacity(), INLINE_CAP);

  let mut compact_immutable = compact::Bytes::from("b".repeat(80));
  compact_immutable.advance(75);
  assert_eq!(
    compact_immutable.try_into_buffer().unwrap().as_slice(),
    b"bbbbb"
  );

  let mut utf8 = Utf8BytesMut::from("a".repeat(INLINE_CAP - 2));
  utf8.try_insert(utf8.len(), '🦀').unwrap();
  assert!(utf8.is_heap());
  assert_eq!(utf8.remove(utf8.len() - '🦀'.len_utf8()), '🦀');

  let mut promoted = Utf8BytesMut::from(format!("{}é", "a".repeat(INLINE_CAP - 2)));
  assert!(promoted.is_inline());
  promoted
    .try_extend_from_within(INLINE_CAP - 2..INLINE_CAP)
    .unwrap();
  assert!(promoted.is_heap());
  assert_eq!(
    promoted.as_str(),
    format!("{}éé", "a".repeat(INLINE_CAP - 2))
  );

  let start_calls = Cell::new(0);
  let end_calls = Cell::new(0);
  let range = CountingRange {
    start: 1,
    end: 3,
    start_calls: &start_calls,
    end_calls: &end_calls,
  };
  let mut counted_utf8 = Utf8BytesMut::from("aé");
  counted_utf8.try_extend_from_within(range).unwrap();
  assert_eq!(start_calls.get(), 1);
  assert_eq!(end_calls.get(), 1);
  assert_eq!(counted_utf8.as_str(), "aéé");

  let mut reclaimed = Utf8BytesMut::from("a".repeat(INLINE_CAP));
  let _ = reclaimed.split_to(4);
  assert_eq!(reclaimed.capacity(), INLINE_CAP - 4);
  reclaimed.try_extend_from_within(0..4).unwrap();
  assert!(reclaimed.is_inline());
  assert_eq!(reclaimed.capacity(), INLINE_CAP);
  assert_eq!(reclaimed.as_str(), "a".repeat(INLINE_CAP));

  let mut text = Utf8BytesMut::with_capacity(128);
  text.push_str("prefix-é");
  let prefix = text.split_to("prefix-".len());
  let fixed = text.try_into_buffer().unwrap();
  assert_eq!(fixed.as_str(), "é");
  assert_eq!(fixed.capacity(), INLINE_CAP);
  assert_eq!(prefix.as_str(), "prefix-");

  let mut inline_utf = Utf8Bytes::from("abcdef");
  inline_utf.advance(2);
  let fixed = inline_utf.try_into_buffer().unwrap();
  assert_eq!(fixed.as_str(), "cdef");
  assert_eq!(fixed.capacity(), INLINE_CAP - 2);

  let mut shared = Utf8Bytes::from("a".repeat(80));
  shared.advance(75);
  let fixed = shared.try_into_buffer().unwrap();
  assert_eq!(fixed.as_str(), "aaaaa");

  let mut compact = compact::Utf8Bytes::from("b".repeat(80));
  compact.advance(75);
  assert_eq!(compact.try_into_buffer().unwrap().as_str(), "bbbbb");

  let shared_oversized = Utf8Bytes::from("s".repeat(INLINE_CAP + 1));
  let pointer = shared_oversized.as_str().as_ptr();
  let shared_oversized = shared_oversized.try_into_buffer().unwrap_err();
  assert!(shared_oversized.is_heap());
  assert_eq!(shared_oversized.as_str().as_ptr(), pointer);

  let compact_oversized = compact::Utf8Bytes::from("c".repeat(INLINE_CAP + 1));
  let pointer = compact_oversized.as_str().as_ptr();
  let compact_oversized = compact_oversized.try_into_buffer().unwrap_err();
  assert!(compact_oversized.is_heap());
  assert_eq!(compact_oversized.as_str().as_ptr(), pointer);

  let mut compact_utf_source = Utf8BytesMut::with_capacity(128);
  compact_utf_source.push_str("é");
  let compact_utf = compact_utf_source.freeze_compact();
  assert!(compact_utf.is_heap());
  let fixed = compact_utf.try_into_buffer().unwrap();
  assert_eq!(fixed.as_str(), "é");
  assert_eq!(fixed.capacity(), INLINE_CAP);

  let mut mutable_oversized = Utf8BytesMut::with_capacity(128);
  mutable_oversized.push_str("m".repeat(INLINE_CAP + 1).as_str());
  let pointer = mutable_oversized.as_str().as_ptr();
  let capacity = mutable_oversized.capacity();
  let mutable_oversized = mutable_oversized.try_into_buffer().unwrap_err();
  assert!(mutable_oversized.is_heap());
  assert_eq!(mutable_oversized.as_str().as_ptr(), pointer);
  assert_eq!(mutable_oversized.capacity(), capacity);
}

#[test]
fn range_errors_stay_typed_for_byte_and_utf8_extensions() {
  let mut bytes = Buffer::from(*b"abc");
  assert_eq!(
    bytes.try_extend_from_within(..=usize::MAX),
    Err(ByteEditError::Range(RangeOutOfBounds::new(
      0,
      usize::MAX,
      3
    )))
  );

  let mut text = Utf8Buffer::try_from("é").unwrap();
  assert_eq!(
    text.try_extend_from_within(1..2),
    Err(Utf8EditError::Index(Utf8IndexError::InvalidCharBoundary {
      index: 1,
    }))
  );
}
