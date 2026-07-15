#![warn(rust_2018_idioms)]

use core::ops::Bound;
use smol_bytes::{Buffer, OutOfBounds, INLINE_CAP};

#[test]
fn buffer_layout_is_unchanged() {
  assert_eq!(core::mem::size_of::<Buffer>(), INLINE_CAP + 2);
}

#[test]
fn advanced_len_truncate_and_empty_are_visible() {
  let mut buffer = Buffer::from(*b"abcdef");
  buffer.advance(2);

  assert_eq!(buffer.len(), 4);
  assert_eq!(buffer.remaining(), 4);
  assert!(!buffer.is_empty());
  assert_eq!(buffer.capacity(), INLINE_CAP - 2);

  // This source/destination combination overlapped in the old truncate copy.
  buffer.truncate(3);
  assert_eq!(buffer.as_slice(), b"cde");
  assert_eq!(buffer.len(), 3);
  assert_eq!(buffer.capacity(), INLINE_CAP - 2);

  buffer.truncate(usize::MAX);
  assert_eq!(buffer.as_slice(), b"cde");

  buffer.truncate(0);
  assert!(buffer.is_empty());
  assert_eq!(buffer.capacity(), INLINE_CAP - 2);
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn fallible_slice_uses_visible_checked_ranges() {
  let mut buffer = Buffer::from(*b"abcdef");
  buffer.advance(2);

  assert_eq!(buffer.try_slice(0..4).unwrap().as_slice(), b"cdef");
  assert!(buffer.try_slice(4..5).is_err());
  assert!(buffer.try_slice(3..2).is_err());
  assert!(buffer.try_slice(..=usize::MAX).is_err());
  assert!(buffer
    .try_slice((Bound::Excluded(usize::MAX), Bound::Unbounded))
    .is_err());
}

#[test]
fn advanced_resize_is_relative_to_the_visible_view() {
  let mut buffer = Buffer::from(*b"abcdef");
  buffer.advance(2);

  buffer.resize(6);
  assert_eq!(buffer.as_slice(), b"cdef\0\0");
  assert_eq!(buffer.len(), 6);

  buffer.resize(1);
  assert_eq!(buffer.as_slice(), b"c");
  assert_eq!(buffer.len(), 1);

  let mut fallible = Buffer::from(*b"abcdef");
  fallible.advance(2);
  fallible.try_resize(6).unwrap();
  assert_eq!(fallible.as_slice(), b"cdef\0\0");
  assert_eq!(fallible.len(), 6);

  assert_eq!(
    buffer.try_resize(buffer.capacity() + 1),
    Err(OutOfBounds::new(buffer.capacity() + 1, buffer.capacity()))
  );
  assert_eq!(buffer.as_slice(), b"c");
}

#[test]
fn advanced_set_len_uses_a_visible_length() {
  let mut buffer = Buffer::from(*b"abcdef");
  buffer.advance(2);

  // SAFETY: `cdef` was initialized by construction and both requested
  // visible lengths are within the current capacity.
  unsafe { buffer.set_len(2) };
  assert_eq!(buffer.as_slice(), b"cd");

  // SAFETY: the original initialized bytes through `f` remain initialized.
  unsafe { buffer.set_len(4) };
  assert_eq!(buffer.as_slice(), b"cdef");
}

#[test]
fn advanced_splits_are_bounded_by_visible_length() {
  let mut buffer = Buffer::from(*b"abcdef");
  buffer.advance(2);

  assert_eq!(buffer.try_split_to(5), Err(OutOfBounds::new(5, 4)));
  let head = buffer.try_split_to(2).unwrap();
  assert_eq!(head.as_slice(), b"cd");
  assert_eq!(buffer.as_slice(), b"ef");

  assert_eq!(buffer.try_split_off(3), Err(OutOfBounds::new(3, 2)));
}

#[test]
fn fallible_signed_reads_sign_extend_and_advance_like_infallible_reads() {
  const BIG_ENDIAN: [u8; 3] = [0xff, 0x00, 0x01];
  const LITTLE_ENDIAN: [u8; 3] = [0x01, 0x00, 0xff];

  macro_rules! assert_signed_read_parity {
    ($input:expr, $get:ident, $try_get:ident) => {{
      let mut infallible = Buffer::from($input);
      let mut fallible = Buffer::from($input);

      assert_eq!(infallible.$get(3), -65_535);
      assert_eq!(fallible.$try_get(3), Ok(-65_535));
      assert_eq!(infallible.remaining(), 0);
      assert_eq!(fallible.remaining(), 0);
    }};
  }

  assert_signed_read_parity!(BIG_ENDIAN, get_int, try_get_int);
  assert_signed_read_parity!(LITTLE_ENDIAN, get_int_le, try_get_int_le);

  let native_endian = if cfg!(target_endian = "big") {
    BIG_ENDIAN
  } else {
    LITTLE_ENDIAN
  };
  assert_signed_read_parity!(native_endian, get_int_ne, try_get_int_ne);

  let mut short = Buffer::from([0xff, 0x00]);
  assert!(short.try_get_int(3).is_err());
  assert_eq!(short.as_slice(), &[0xff, 0x00]);
}

#[test]
fn signed_variable_width_edge_reads_preserve_parity_and_cursor() {
  const EMPTY: [u8; 0] = [];
  const NEGATIVE: [u8; 1] = [0x80];
  const BIG_ENDIAN_BOUNDARY: [u8; 8] = [0x80, 0, 0, 0, 0, 0, 0, 0];
  const LITTLE_ENDIAN_BOUNDARY: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0x80];

  macro_rules! assert_zero_width_read {
    ($get:ident, $try_get:ident) => {{
      let mut infallible = Buffer::from(EMPTY);
      let mut fallible = Buffer::from(EMPTY);
      let infallible_remaining = infallible.remaining();
      let fallible_remaining = fallible.remaining();

      assert_eq!(infallible.$get(0), 0);
      assert_eq!(fallible.$try_get(0), Ok(0));
      assert_eq!(infallible.remaining(), infallible_remaining);
      assert_eq!(fallible.remaining(), fallible_remaining);
    }};
  }

  macro_rules! assert_nonzero_width_read {
    ($input:expr, $nbytes:expr, $expected:expr, $get:ident, $try_get:ident) => {{
      let mut infallible = Buffer::from($input);
      let mut fallible = Buffer::from($input);
      let infallible_remaining = infallible.remaining();
      let fallible_remaining = fallible.remaining();

      assert_eq!(infallible.$get($nbytes), $expected);
      assert_eq!(fallible.$try_get($nbytes), Ok($expected));
      assert_eq!(infallible.remaining(), infallible_remaining - $nbytes);
      assert_eq!(fallible.remaining(), fallible_remaining - $nbytes);
    }};
  }

  assert_zero_width_read!(get_int, try_get_int);
  assert_zero_width_read!(get_int_le, try_get_int_le);
  assert_zero_width_read!(get_int_ne, try_get_int_ne);

  assert_nonzero_width_read!(NEGATIVE, 1, -128, get_int, try_get_int);
  assert_nonzero_width_read!(NEGATIVE, 1, -128, get_int_le, try_get_int_le);
  assert_nonzero_width_read!(NEGATIVE, 1, -128, get_int_ne, try_get_int_ne);

  let native_endian_boundary = if cfg!(target_endian = "big") {
    BIG_ENDIAN_BOUNDARY
  } else {
    LITTLE_ENDIAN_BOUNDARY
  };
  assert_nonzero_width_read!(BIG_ENDIAN_BOUNDARY, 8, i64::MIN, get_int, try_get_int);
  assert_nonzero_width_read!(
    LITTLE_ENDIAN_BOUNDARY,
    8,
    i64::MIN,
    get_int_le,
    try_get_int_le
  );
  assert_nonzero_width_read!(
    native_endian_boundary,
    8,
    i64::MIN,
    get_int_ne,
    try_get_int_ne
  );
}
