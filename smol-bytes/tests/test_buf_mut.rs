#![warn(rust_2018_idioms)]

use bytes::buf::UninitSlice;
use core::fmt::Write;
use core::mem::MaybeUninit;
use smol_bytes::{Buf, BufMut, Buffer, BytesMut, INLINE_CAP};

#[test]
fn test_vec_as_mut_buf() {
  let mut buf = Vec::with_capacity(64);

  assert_eq!(buf.remaining_mut(), isize::MAX as usize);

  assert!(buf.chunk_mut().len() >= 64);

  buf.put(&b"zomg"[..]);

  assert_eq!(&buf, b"zomg");

  assert_eq!(buf.remaining_mut(), isize::MAX as usize - 4);
  assert_eq!(buf.capacity(), 64);

  for _ in 0..16 {
    buf.put(&b"zomg"[..]);
  }

  assert_eq!(buf.len(), 68);
}

#[test]
fn test_vec_put_bytes() {
  let mut buf = Vec::new();
  buf.push(17);
  buf.put_bytes(19, 2);
  assert_eq!([17, 19, 19], &buf[..]);
}

#[test]
fn test_put_u8() {
  let mut buf = Vec::with_capacity(8);
  buf.put_u8(33);
  assert_eq!(b"\x21", &buf[..]);
}

#[test]
fn test_put_u16() {
  let mut buf = Vec::with_capacity(8);
  buf.put_u16(8532);
  assert_eq!(b"\x21\x54", &buf[..]);

  buf.clear();
  buf.put_u16_le(8532);
  assert_eq!(b"\x54\x21", &buf[..]);
}

#[test]
fn test_put_int() {
  let mut buf = Vec::with_capacity(8);
  buf.put_int(0x1020304050607080, 3);
  assert_eq!(b"\x60\x70\x80", &buf[..]);
}

#[test]
#[should_panic]
fn test_put_int_nbytes_overflow() {
  let mut buf = Vec::with_capacity(8);
  buf.put_int(0x1020304050607080, 9);
}

#[test]
fn test_put_int_le() {
  let mut buf = Vec::with_capacity(8);
  buf.put_int_le(0x1020304050607080, 3);
  assert_eq!(b"\x80\x70\x60", &buf[..]);
}

#[test]
#[should_panic]
fn test_put_int_le_nbytes_overflow() {
  let mut buf = Vec::with_capacity(8);
  buf.put_int_le(0x1020304050607080, 9);
}

#[test]
#[should_panic(expected = "advance out of bounds: the len is 8 but advancing by 12")]
fn test_vec_advance_mut() {
  // Verify fix for #354
  let mut buf = Vec::with_capacity(8);
  unsafe {
    buf.advance_mut(12);
  }
}

#[test]
fn test_clone() {
  let mut buf = BytesMut::with_capacity(100);
  buf.write_str("this is a test").unwrap();
  let buf2 = buf.clone();

  buf.write_str(" of our emergency broadcast system").unwrap();
  assert!(buf != buf2);
}

fn do_test_slice_small<T: ?Sized>(make: impl Fn(&mut [u8]) -> &mut T)
where
  for<'r> &'r mut T: BufMut,
{
  let mut buf = [b'X'; 8];

  let mut slice = make(&mut buf[..]);
  slice.put_bytes(b'A', 2);
  slice.put_u8(b'B');
  slice.put_slice(b"BCC");
  assert_eq!(2, slice.remaining_mut());
  assert_eq!(b"AABBCCXX", &buf[..]);

  let mut slice = make(&mut buf[..]);
  slice.put_u32(0x61626364);
  assert_eq!(4, slice.remaining_mut());
  assert_eq!(b"abcdCCXX", &buf[..]);

  let mut slice = make(&mut buf[..]);
  slice.put_u32_le(0x30313233);
  assert_eq!(4, slice.remaining_mut());
  assert_eq!(b"3210CCXX", &buf[..]);
}

fn do_test_slice_large<T: ?Sized>(make: impl Fn(&mut [u8]) -> &mut T)
where
  for<'r> &'r mut T: BufMut,
{
  const LEN: usize = 100;
  const FILL: [u8; LEN] = [b'Y'; LEN];

  let test = |fill: &dyn Fn(&mut &mut T, usize)| {
    for buf_len in 0..LEN {
      let mut buf = [b'X'; LEN];
      for fill_len in 0..=buf_len {
        let mut slice = make(&mut buf[..buf_len]);
        fill(&mut slice, fill_len);
        assert_eq!(buf_len - fill_len, slice.remaining_mut());
        let (head, tail) = buf.split_at(fill_len);
        assert_eq!(&FILL[..fill_len], head);
        assert!(tail.iter().all(|b| *b == b'X'));
      }
    }
  };

  test(&|slice, fill_len| slice.put_slice(&FILL[..fill_len]));
  test(&|slice, fill_len| slice.put_bytes(FILL[0], fill_len));
}

fn do_test_slice_put_slice_panics<T: ?Sized>(make: impl Fn(&mut [u8]) -> &mut T)
where
  for<'r> &'r mut T: BufMut,
{
  let mut buf = [b'X'; 4];
  let mut slice = make(&mut buf[..]);
  slice.put_slice(b"12345");
}

fn do_test_slice_put_bytes_panics<T: ?Sized>(make: impl Fn(&mut [u8]) -> &mut T)
where
  for<'r> &'r mut T: BufMut,
{
  let mut buf = [b'X'; 4];
  let mut slice = make(&mut buf[..]);
  slice.put_bytes(b'1', 5);
}

#[test]
fn test_slice_buf_mut_small() {
  do_test_slice_small(|x| x);
}

#[test]
fn test_slice_buf_mut_large() {
  do_test_slice_large(|x| x);
}

#[test]
#[should_panic]
fn test_slice_buf_mut_put_slice_overflow() {
  do_test_slice_put_slice_panics(|x| x);
}

#[test]
#[should_panic]
fn test_slice_buf_mut_put_bytes_overflow() {
  do_test_slice_put_bytes_panics(|x| x);
}

fn make_maybe_uninit_slice(slice: &mut [u8]) -> &mut [MaybeUninit<u8>] {
  // SAFETY: [u8] has the same layout as [MaybeUninit<u8>].
  unsafe { core::mem::transmute(slice) }
}

#[test]
fn test_maybe_uninit_buf_mut_small() {
  do_test_slice_small(make_maybe_uninit_slice);
}

#[test]
fn test_maybe_uninit_buf_mut_large() {
  do_test_slice_large(make_maybe_uninit_slice);
}

#[test]
#[should_panic]
fn test_maybe_uninit_buf_mut_put_slice_overflow() {
  do_test_slice_put_slice_panics(make_maybe_uninit_slice);
}

#[test]
#[should_panic]
fn test_maybe_uninit_buf_mut_put_bytes_overflow() {
  do_test_slice_put_bytes_panics(make_maybe_uninit_slice);
}

#[allow(unused_allocation)] // This is intentional.
#[test]
fn test_deref_bufmut_forwards() {
  struct Special;

  unsafe impl BufMut for Special {
    fn remaining_mut(&self) -> usize {
      unreachable!("remaining_mut");
    }

    fn chunk_mut(&mut self) -> &mut UninitSlice {
      unreachable!("chunk_mut");
    }

    unsafe fn advance_mut(&mut self, _: usize) {
      unreachable!("advance");
    }

    fn put_u8(&mut self, _: u8) {
      // specialized!
    }
  }

  // these should all use the specialized method
  Special.put_u8(b'x');
  (&mut Special as &mut dyn BufMut).put_u8(b'x');
  (Box::new(Special) as Box<dyn BufMut>).put_u8(b'x');
  Box::new(Special).put_u8(b'x');
}

#[test]
#[should_panic]
fn write_byte_panics_if_out_of_bounds() {
  let mut data = *b"bar";

  let slice = unsafe { UninitSlice::from_raw_parts_mut(data.as_mut_ptr(), 3) };
  slice.write_byte(4, b'f');
}

fn inline_full() -> BytesMut {
  BytesMut::from(&[b'a'; INLINE_CAP])
}

fn heap_with_data() -> BytesMut {
  let mut buffer = BytesMut::with_capacity(128);
  buffer.put_slice(b"abcdef");
  buffer
}

#[test]
fn advanced_bytes_mut_len_resize_set_len_and_split_are_visible() {
  for mut buffer in [BytesMut::from(&b"abcdef"[..]), heap_with_data()] {
    buffer.advance(2);
    assert_eq!(buffer.len(), 4);
    assert_eq!(buffer.remaining(), 4);
    assert!(!buffer.is_empty());

    assert_eq!(
      buffer.try_split_to(5),
      Err(smol_bytes::OutOfBounds::new(5, 4))
    );

    buffer.resize(6, b'x');
    assert_eq!(buffer.as_slice(), b"cdefxx");
    assert_eq!(buffer.len(), 6);

    // SAFETY: shrinking the visible view exposes no uninitialized bytes.
    unsafe { buffer.set_len(2) };
    assert_eq!(buffer.as_slice(), b"cd");

    let head = buffer
      .try_split_to(2)
      .unwrap()
      .unwrap_or_else(BytesMut::from);
    assert_eq!(head.as_slice(), b"cd");
    assert!(buffer.is_empty());
  }
}

#[test]
fn inline_growth_reclaims_consumed_prefix_on_demand() {
  let mut reserved = inline_full();
  reserved.advance(8);
  assert_eq!(reserved.capacity(), INLINE_CAP - 8);
  reserved.reserve(8);
  assert!(reserved.is_inline());
  assert_eq!(reserved.capacity(), INLINE_CAP);
  assert_eq!(reserved.as_slice(), &[b'a'; INLINE_CAP - 8]);

  let mut extended = inline_full();
  extended.advance(8);
  extended.extend_from_slice(&[b'b'; 8]);
  assert!(extended.is_inline());
  assert_eq!(extended.len(), INLINE_CAP);
  assert_eq!(&extended[INLINE_CAP - 8..], &[b'b'; 8]);

  let mut resized = inline_full();
  resized.advance(8);
  resized.resize(INLINE_CAP, b'c');
  assert!(resized.is_inline());
  assert_eq!(&resized[INLINE_CAP - 8..], &[b'c'; 8]);

  let mut filled = inline_full();
  filled.advance(8);
  filled.put_bytes(b'd', 8);
  assert!(filled.is_inline());
  assert_eq!(&filled[INLINE_CAP - 8..], &[b'd'; 8]);

  let mut reclaimed = inline_full();
  reclaimed.advance(8);
  assert!(reclaimed.try_reclaim(8));
  assert!(reclaimed.is_inline());
  assert_eq!(reclaimed.capacity(), INLINE_CAP);
  assert_eq!(reclaimed.as_slice(), &[b'a'; INLINE_CAP - 8]);
}

#[test]
fn bytes_mut_chunk_mut_makes_progress_when_inline_tail_is_full() {
  let mut full_for_chunk = inline_full();
  assert!(full_for_chunk.remaining_mut() > 0);
  let (first_ptr, first_len) = {
    let chunk = full_for_chunk.chunk_mut();
    assert!(chunk.len() >= 64);
    (chunk.as_mut_ptr(), chunk.len())
  };
  let capacity_after_promotion = full_for_chunk.capacity();
  assert!(!full_for_chunk.is_inline());
  assert_eq!(full_for_chunk.as_slice(), &[b'a'; INLINE_CAP]);

  let repeated_ptr = {
    let chunk = full_for_chunk.chunk_mut();
    assert_eq!(chunk.len(), first_len);
    chunk.write_byte(0, b'!');
    chunk.as_mut_ptr()
  };
  assert_eq!(repeated_ptr, first_ptr);
  assert_eq!(full_for_chunk.capacity(), capacity_after_promotion);

  // SAFETY: `write_byte` above initialized the one byte being advanced.
  unsafe { full_for_chunk.advance_mut(1) };
  let mut expected = vec![b'a'; INLINE_CAP];
  expected.push(b'!');
  assert_eq!(full_for_chunk.as_slice(), expected.as_slice());

  let post_progress_ptr = full_for_chunk.chunk_mut().as_mut_ptr();
  let post_progress_capacity = full_for_chunk.capacity();
  assert_eq!(full_for_chunk.chunk_mut().as_mut_ptr(), post_progress_ptr);
  assert_eq!(full_for_chunk.capacity(), post_progress_capacity);

  let mut advanced_for_chunk = inline_full();
  advanced_for_chunk.advance(1);
  assert!(advanced_for_chunk.remaining_mut() > 0);
  assert!(advanced_for_chunk.chunk_mut().len() > 0);
  assert!(advanced_for_chunk.is_inline());
  assert_eq!(advanced_for_chunk.as_slice(), &[b'a'; INLINE_CAP - 1]);

  let mut full_for_put = inline_full();
  full_for_put.put(&b"!!"[..]);
  let mut expected = vec![b'a'; INLINE_CAP];
  expected.extend_from_slice(b"!!");
  assert_eq!(full_for_put.as_slice(), expected.as_slice());
  assert!(!full_for_put.is_inline());

  let mut advanced_for_put = inline_full();
  advanced_for_put.advance(1);
  advanced_for_put.put(&b"!"[..]);
  let mut expected = vec![b'a'; INLINE_CAP - 1];
  expected.push(b'!');
  assert_eq!(advanced_for_put.as_slice(), expected.as_slice());
  assert!(advanced_for_put.is_inline());
}

#[test]
#[cfg_attr(not(panic = "unwind"), ignore)]
fn reserve_and_put_bytes_overflow_panic_before_mutation() {
  use std::panic::{AssertUnwindSafe, catch_unwind};

  for mut reserved in [BytesMut::from(&b"x"[..]), heap_with_data()] {
    let before = reserved.as_slice().to_vec();
    assert!(catch_unwind(AssertUnwindSafe(|| reserved.reserve(usize::MAX))).is_err());
    assert_eq!(reserved.as_slice(), before);
  }

  for mut filled in [BytesMut::from(&b"x"[..]), heap_with_data()] {
    let before = filled.as_slice().to_vec();
    assert!(catch_unwind(AssertUnwindSafe(|| filled.put_bytes(0, usize::MAX))).is_err());
    assert_eq!(filled.as_slice(), before);
  }
}

#[test]
#[should_panic]
fn copy_from_slice_panics_if_different_length_1() {
  let mut data = *b"bar";

  let slice = unsafe { UninitSlice::from_raw_parts_mut(data.as_mut_ptr(), 3) };
  slice.copy_from_slice(b"a");
}

#[test]
#[should_panic]
fn copy_from_slice_panics_if_different_length_2() {
  let mut data = *b"bar";

  let slice = unsafe { UninitSlice::from_raw_parts_mut(data.as_mut_ptr(), 3) };
  slice.copy_from_slice(b"abcd");
}

// ---------------------------------------------------------------------------
// Forwarded `BufMut` writers, verified by reading the value back through `Buf`.
// The generic `B: Buf + BufMut` bound forces trait-method (macro-forwarded)
// dispatch for both the writes and the read-back.
// ---------------------------------------------------------------------------

fn check_writes<B: Buf + BufMut>(fresh: impl Fn() -> B) {
  macro_rules! wr {
    ($put:ident, $put_le:ident, $put_ne:ident, $get:ident, $get_le:ident, $get_ne:ident,
     $be:expr, $le:expr, $val:expr) => {{
      let val = $val;
      // Big-endian: exact byte pattern, then value roundtrip.
      let mut b = fresh();
      b.$put(val);
      assert_eq!(b.chunk(), &$be[..]);
      assert_eq!(b.$get(), val);
      // Little-endian: exact byte pattern, then value roundtrip.
      let mut b = fresh();
      b.$put_le(val);
      assert_eq!(b.chunk(), &$le[..]);
      assert_eq!(b.$get_le(), val);
      // Native-endian: value roundtrip.
      let mut b = fresh();
      b.$put_ne(val);
      assert_eq!(b.$get_ne(), val);
    }};
  }

  wr!(
    put_u16,
    put_u16_le,
    put_u16_ne,
    get_u16,
    get_u16_le,
    get_u16_ne,
    [0x01u8, 0x02],
    [0x02u8, 0x01],
    0x0102u16
  );
  wr!(
    put_i16,
    put_i16_le,
    put_i16_ne,
    get_i16,
    get_i16_le,
    get_i16_ne,
    [0xFFu8, 0xFE],
    [0xFEu8, 0xFF],
    -2i16
  );
  wr!(
    put_u32,
    put_u32_le,
    put_u32_ne,
    get_u32,
    get_u32_le,
    get_u32_ne,
    [0x01u8, 0x02, 0x03, 0x04],
    [0x04u8, 0x03, 0x02, 0x01],
    0x01020304u32
  );
  wr!(
    put_i32,
    put_i32_le,
    put_i32_ne,
    get_i32,
    get_i32_le,
    get_i32_ne,
    [0xFFu8, 0xFF, 0xFF, 0xFE],
    [0xFEu8, 0xFF, 0xFF, 0xFF],
    -2i32
  );
  wr!(
    put_u64,
    put_u64_le,
    put_u64_ne,
    get_u64,
    get_u64_le,
    get_u64_ne,
    [0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
    [0x08u8, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
    0x0102030405060708u64
  );
  wr!(
    put_i64,
    put_i64_le,
    put_i64_ne,
    get_i64,
    get_i64_le,
    get_i64_ne,
    [0xFFu8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE],
    [0xFEu8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    -2i64
  );
  wr!(
    put_u128,
    put_u128_le,
    put_u128_ne,
    get_u128,
    get_u128_le,
    get_u128_ne,
    [
      0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
      0x10
    ],
    [
      0x10u8, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02,
      0x01
    ],
    0x0102030405060708090A0B0C0D0E0F10u128
  );
  wr!(
    put_i128,
    put_i128_le,
    put_i128_ne,
    get_i128,
    get_i128_le,
    get_i128_ne,
    [
      0xFFu8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
      0xFE
    ],
    [
      0xFEu8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
      0xFF
    ],
    -2i128
  );
  wr!(
    put_f32,
    put_f32_le,
    put_f32_ne,
    get_f32,
    get_f32_le,
    get_f32_ne,
    [0x41u8, 0x48, 0x00, 0x00],
    [0x00u8, 0x00, 0x48, 0x41],
    12.5f32
  );
  wr!(
    put_f64,
    put_f64_le,
    put_f64_ne,
    get_f64,
    get_f64_le,
    get_f64_ne,
    [0x40u8, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    [0x00u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40],
    12.5f64
  );

  // 8-bit writers.
  let mut b = fresh();
  b.put_u8(0xAB);
  assert_eq!(b.chunk(), &[0xABu8][..]);
  assert_eq!(b.get_u8(), 0xAB);
  let mut b = fresh();
  b.put_i8(-5);
  assert_eq!(b.get_i8(), -5);

  // Variable-width writers, nbytes 1..=8. `v` is chosen to fit in `nbytes`.
  let full = 0x1122334455667788u64;
  for nbytes in 1..=8usize {
    let v = full.to_be_bytes()[8 - nbytes..]
      .iter()
      .fold(0u64, |acc, &b| (acc << 8) | b as u64);

    let mut b = fresh();
    b.put_uint(v, nbytes);
    assert_eq!(b.get_uint(nbytes), v);
    let mut b = fresh();
    b.put_uint_le(v, nbytes);
    assert_eq!(b.get_uint_le(nbytes), v);
    let mut b = fresh();
    b.put_uint_ne(v, nbytes);
    assert_eq!(b.get_uint_ne(nbytes), v);

    let mut b = fresh();
    b.put_int(-1, nbytes);
    assert_eq!(b.get_int(nbytes), -1);
    let mut b = fresh();
    b.put_int_le(-1, nbytes);
    assert_eq!(b.get_int_le(nbytes), -1);
    let mut b = fresh();
    b.put_int_ne(-1, nbytes);
    assert_eq!(b.get_int_ne(nbytes), -1);
  }
}

#[test]
fn forwarded_buf_mut_writes_buffer() {
  check_writes(Buffer::new);
}

#[test]
fn forwarded_buf_mut_writes_bytes_mut() {
  check_writes(BytesMut::new);
}

#[test]
fn buffer_try_put_capacity_and_length_errors() {
  // Exactly at the boundary: 60 used, 2 free -> a u16 fits.
  let mut b = Buffer::try_from(&[0u8; 60][..]).unwrap();
  assert!(b.try_put_u16(0x0102).is_ok());
  assert_eq!(b.len(), 62);

  // One byte short: a u16 no longer fits.
  let mut b = Buffer::try_from(&[0u8; 61][..]).unwrap();
  assert_eq!(
    b.try_put_u16(0x0102),
    Err(smol_bytes::TryPutError {
      requested: 2,
      available: 1,
    })
  );

  // A u32 needs four bytes; only two are free.
  let mut b = Buffer::try_from(&[0u8; 60][..]).unwrap();
  assert!(b.try_put_u32(0x01020304).is_err());
  let mut b = Buffer::try_from(&[0u8; 60][..]).unwrap();
  assert!(b.try_put_f64(1.0).is_err());

  // try_put_uint: NotEnoughSpace vs InvalidLength.
  let mut b = Buffer::new();
  assert!(b.try_put_uint(0x010203, 3).is_ok());
  let mut b = Buffer::try_from(&[0u8; 60][..]).unwrap();
  assert!(matches!(
    b.try_put_uint(0x010203, 3),
    Err(smol_bytes::TryPutIntegerError::NotEnoughSpace(_))
  ));
  let mut b = Buffer::new();
  assert!(matches!(
    b.try_put_uint(0, 9),
    Err(smol_bytes::TryPutIntegerError::InvalidLength(_))
  ));

  // try_put_int mirrors the unsigned behaviour.
  let mut b = Buffer::new();
  assert!(b.try_put_int(-1, 4).is_ok());
  let mut b = Buffer::try_from(&[0u8; 60][..]).unwrap();
  assert!(b.try_put_int(-1, 3).is_err());

  // try_put_slice success and overflow.
  let mut b = Buffer::new();
  assert!(b.try_put_slice(b"hello").is_ok());
  let mut b = Buffer::try_from(&[0u8; 60][..]).unwrap();
  assert!(b.try_put_slice(b"hello").is_err());
}

#[test]
fn write_str_grows_past_inline_capacity() {
  let mut buf = BytesMut::new();
  let first = "a".repeat(INLINE_CAP + 1);
  write!(buf, "{}", first).unwrap();
  assert_eq!(buf.as_slice(), first.as_bytes());

  let second = "b".repeat(50);
  write!(buf, "{}", second).unwrap();

  let mut expected = first.into_bytes();
  expected.extend_from_slice(second.as_bytes());
  assert!(expected.len() > 100);
  assert_eq!(buf.as_slice(), expected.as_slice());
}
