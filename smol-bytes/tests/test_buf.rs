#![warn(rust_2018_idioms)]

use ::smol_bytes::{Buf, Bytes, BytesMut};
use core::{cmp, mem};
use std::collections::VecDeque;
#[cfg(feature = "std")]
use std::io::IoSlice;

// A random 64-byte ascii string, with the first 8 bytes altered to
// give valid representations of f32 and f64 (making them easier to compare)
// and negative signed numbers when interpreting as big endian
// (testing Sign Extension for `Buf::get_int' and `Buf::get_int_ne`).
const INPUT: &[u8] = b"\xffFqrjrDqPhvTc45vvq33f6bJrUtyHESuTeklWKgYd64xgzxJwvAkpYYnpNJyZSRn";

macro_rules! e {
  ($big_endian_val:expr, $little_endian_val:expr) => {
    if cfg!(target_endian = "big") {
      $big_endian_val
    } else {
      $little_endian_val
    }
  };
}

macro_rules! buf_tests {
    ($make_input:ident) => {
        buf_tests!($make_input, true);
    };
    ($make_input:ident, $checks_vectored_is_complete:expr) => {
        use super::*;

        #[test]
        fn empty_state() {
            let buf = $make_input(&[]);
            assert_eq!(buf.remaining(), 0);
            assert!(!buf.has_remaining());
            assert!(buf.chunk().is_empty());
        }

        #[test]
        fn fresh_state() {
            let buf = $make_input(INPUT);
            assert_eq!(buf.remaining(), 64);
            assert!(buf.has_remaining());

            let chunk = buf.chunk();
            assert!(chunk.len() <= 64);
            assert!(INPUT.starts_with(chunk));
        }

        #[test]
        fn advance() {
            let mut buf = $make_input(INPUT);
            buf.advance(8);
            assert_eq!(buf.remaining(), 64 - 8);
            assert!(buf.has_remaining());

            let chunk = buf.chunk();
            assert!(chunk.len() <= 64 - 8);
            assert!(INPUT[8..].starts_with(chunk));
        }

        #[test]
        fn advance_to_end() {
            let mut buf = $make_input(INPUT);
            buf.advance(64);
            assert_eq!(buf.remaining(), 0);
            assert!(!buf.has_remaining());

            let chunk = buf.chunk();
            assert!(chunk.is_empty());
        }

        #[test]
        #[should_panic]
        fn advance_past_end() {
            let  mut buf = $make_input(INPUT);
            buf.advance(65);
        }

        #[test]
        #[cfg(feature = "std")]
        fn chunks_vectored_empty() {
            let  buf = $make_input(&[]);
            let mut bufs = [IoSlice::new(&[]); 16];

            let n = buf.chunks_vectored(&mut bufs);
            assert_eq!(n, 0);
            assert!(bufs.iter().all(|buf| buf.is_empty()));
        }

        #[test]
        #[cfg(feature = "std")]
        fn chunks_vectored_is_complete() {
            let buf = $make_input(INPUT);
            let mut bufs = [IoSlice::new(&[]); 16];

            let n = buf.chunks_vectored(&mut bufs);
            assert!(n > 0);
            assert!(n <= 16);

            let bufs_concat = bufs[..n]
                .iter()
                .flat_map(|b| b.iter().copied())
                .collect::<Vec<u8>>();
            if $checks_vectored_is_complete {
                assert_eq!(bufs_concat, INPUT);
            } else {
                // If this panics then `buf` implements `chunks_vectored`.
                // Remove the `false` argument from `buf_tests!` for that type.
                assert!(bufs_concat.len() < INPUT.len());
                assert!(INPUT.starts_with(&bufs_concat));
            }

            for i in n..16 {
                assert!(bufs[i].is_empty());
            }
        }

        #[test]
        fn copy_to_slice() {
            let mut buf = $make_input(INPUT);

            let mut chunk = [0u8; 8];
            buf.copy_to_slice(&mut chunk);
            assert_eq!(buf.remaining(), 64 - 8);
            assert!(buf.has_remaining());
            assert_eq!(chunk, INPUT[..8]);

            let chunk = buf.chunk();
            assert!(chunk.len() <= 64 - 8);
            assert!(INPUT[8..].starts_with(chunk));
        }

        #[test]
        fn copy_to_slice_big() {
            let mut buf = $make_input(INPUT);

            let mut chunk = [0u8; 56];
            buf.copy_to_slice(&mut chunk);
            assert_eq!(buf.remaining(), 64 - 56);
            assert!(buf.has_remaining());
            assert_eq!(chunk, INPUT[..56]);

            let chunk = buf.chunk();
            assert!(chunk.len() <= 64 - 56);
            assert!(INPUT[56..].starts_with(chunk));
        }

        #[test]
        fn copy_to_slice_to_end() {
            let mut buf = $make_input(INPUT);

            let mut chunk = [0u8; 64];
            buf.copy_to_slice(&mut chunk);
            assert_eq!(buf.remaining(), 0);
            assert!(!buf.has_remaining());
            assert_eq!(chunk, INPUT);

            assert!(buf.chunk().is_empty());
        }

        #[test]
        #[should_panic]
        fn copy_to_slice_overflow() {
            let mut buf = $make_input(INPUT);

            let mut chunk = [0u8; 65];
            buf.copy_to_slice(&mut chunk);
        }

        #[test]
        fn copy_to_bytes() {
            let mut buf = $make_input(INPUT);

            let chunk = buf.copy_to_bytes(8);
            assert_eq!(buf.remaining(), 64 - 8);
            assert!(buf.has_remaining());
            assert_eq!(chunk, INPUT[..8]);

            let chunk = buf.chunk();
            assert!(chunk.len() <= 64 - 8);
            assert!(INPUT[8..].starts_with(chunk));
        }

        #[test]
        fn copy_to_bytes_big() {
            let mut buf = $make_input(INPUT);

            let chunk = buf.copy_to_bytes(56);
            assert_eq!(buf.remaining(), 64 - 56);
            assert!(buf.has_remaining());
            assert_eq!(chunk, INPUT[..56]);

            let chunk = buf.chunk();
            assert!(chunk.len() <= 64 - 56);
            assert!(INPUT[56..].starts_with(chunk));
        }

        #[test]
        fn copy_to_bytes_to_end() {
            let mut buf = $make_input(INPUT);

            let chunk = buf.copy_to_bytes(64);
            assert_eq!(buf.remaining(), 0);
            assert!(!buf.has_remaining());
            assert_eq!(chunk, INPUT);

            assert!(buf.chunk().is_empty());
        }

        #[test]
        #[should_panic]
        fn copy_to_bytes_overflow() {
            let mut buf = $make_input(INPUT);

            let _ = buf.copy_to_bytes(65);
        }

        buf_tests!(number $make_input, get_u8, get_u8_overflow, u8, get_u8, 0xff);
        buf_tests!(number $make_input, get_i8, get_i8_overflow, i8, get_i8, 0xffu8 as i8);
        buf_tests!(number $make_input, get_u16_be, get_u16_be_overflow, u16, get_u16, 0xff46);
        buf_tests!(number $make_input, get_u16_le, get_u16_le_overflow, u16, get_u16_le, 0x46ff);
        buf_tests!(number $make_input, get_u16_ne, get_u16_ne_overflow, u16, get_u16_ne, e!(0xff46, 0x46ff));
        buf_tests!(number $make_input, get_i16_be, get_i16_be_overflow, i16, get_i16, 0xff46u16 as i16);
        buf_tests!(number $make_input, get_i16_le, get_i16_le_overflow, i16, get_i16_le, 0x46ff);
        buf_tests!(number $make_input, get_i16_ne, get_i16_ne_overflow, i16, get_i16_ne, e!(0xff46u16 as i16, 0x46ff));
        buf_tests!(number $make_input, get_u32_be, get_u32_be_overflow, u32, get_u32, 0xff467172);
        buf_tests!(number $make_input, get_u32_le, get_u32_le_overflow, u32, get_u32_le, 0x727146ff);
        buf_tests!(number $make_input, get_u32_ne, get_u32_ne_overflow, u32, get_u32_ne, e!(0xff467172, 0x727146ff));
        buf_tests!(number $make_input, get_i32_be, get_i32_be_overflow, i32, get_i32, 0xff467172u32 as i32);
        buf_tests!(number $make_input, get_i32_le, get_i32_le_overflow, i32, get_i32_le, 0x727146ff);
        buf_tests!(number $make_input, get_i32_ne, get_i32_ne_overflow, i32, get_i32_ne, e!(0xff467172u32 as i32, 0x727146ff));
        buf_tests!(number $make_input, get_u64_be, get_u64_be_overflow, u64, get_u64, 0xff4671726a724471);
        buf_tests!(number $make_input, get_u64_le, get_u64_le_overflow, u64, get_u64_le, 0x7144726a727146ff);
        buf_tests!(number $make_input, get_u64_ne, get_u64_ne_overflow, u64, get_u64_ne, e!(0xff4671726a724471, 0x7144726a727146ff));
        buf_tests!(number $make_input, get_i64_be, get_i64_be_overflow, i64, get_i64, 0xff4671726a724471u64 as i64);
        buf_tests!(number $make_input, get_i64_le, get_i64_le_overflow, i64, get_i64_le, 0x7144726a727146ff);
        buf_tests!(number $make_input, get_i64_ne, get_i64_ne_overflow, i64, get_i64_ne, e!(0xff4671726a724471u64 as i64, 0x7144726a727146ff));
        buf_tests!(number $make_input, get_u128_be, get_u128_be_overflow, u128, get_u128, 0xff4671726a7244715068765463343576);
        buf_tests!(number $make_input, get_u128_le, get_u128_le_overflow, u128, get_u128_le, 0x76353463547668507144726a727146ff);
        buf_tests!(number $make_input, get_u128_ne, get_u128_ne_overflow, u128, get_u128_ne, e!(0xff4671726a7244715068765463343576, 0x76353463547668507144726a727146ff));
        buf_tests!(number $make_input, get_i128_be, get_i128_be_overflow, i128, get_i128, 0xff4671726a7244715068765463343576u128 as i128);
        buf_tests!(number $make_input, get_i128_le, get_i128_le_overflow, i128, get_i128_le, 0x76353463547668507144726a727146ff);
        buf_tests!(number $make_input, get_i128_ne, get_i128_ne_overflow, i128, get_i128_ne, e!(0xff4671726a7244715068765463343576u128 as i128, 0x76353463547668507144726a727146ff));
        buf_tests!(number $make_input, get_f32_be, get_f32_be_overflow, f32, get_f32, f32::from_bits(0xff467172));
        buf_tests!(number $make_input, get_f32_le, get_f32_le_overflow, f32, get_f32_le, f32::from_bits(0x727146ff));
        buf_tests!(number $make_input, get_f32_ne, get_f32_ne_overflow, f32, get_f32_ne, f32::from_bits(e!(0xff467172, 0x727146ff)));
        buf_tests!(number $make_input, get_f64_be, get_f64_be_overflow, f64, get_f64, f64::from_bits(0xff4671726a724471));
        buf_tests!(number $make_input, get_f64_le, get_f64_le_overflow, f64, get_f64_le, f64::from_bits(0x7144726a727146ff));
        buf_tests!(number $make_input, get_f64_ne, get_f64_ne_overflow, f64, get_f64_ne, f64::from_bits(e!(0xff4671726a724471, 0x7144726a727146ff)));

        buf_tests!(var_number $make_input, get_uint_be, get_uint_be_overflow, u64, get_uint, 3, 0xff4671);
        buf_tests!(var_number $make_input, get_uint_le, get_uint_le_overflow, u64, get_uint_le, 3, 0x7146ff);
        buf_tests!(var_number $make_input, get_uint_ne, get_uint_ne_overflow, u64, get_uint_ne, 3, e!(0xff4671, 0x7146ff));
        buf_tests!(var_number $make_input, get_int_be, get_int_be_overflow, i64, get_int, 3, 0xffffffffffff4671u64 as i64);
        buf_tests!(var_number $make_input, get_int_le, get_int_le_overflow, i64, get_int_le, 3, 0x7146ff);
        buf_tests!(var_number $make_input, get_int_ne, get_int_ne_overflow, i64, get_int_ne, 3, e!(0xffffffffffff4671u64 as i64, 0x7146ff));
    };
    (number $make_input:ident, $ok_name:ident, $panic_name:ident, $number:ty, $method:ident, $value:expr) => {
        #[test]
        fn $ok_name() {
            let mut buf = $make_input(INPUT);

            let value = buf.$method();
            assert_eq!(buf.remaining(), 64 - mem::size_of::<$number>());
            assert!(buf.has_remaining());
            assert_eq!(value, $value);
        }

        #[test]
        #[should_panic]
        fn $panic_name() {
            let mut buf = $make_input(&[]);

            let _ = buf.$method();
        }
    };
    (var_number $make_input:ident, $ok_name:ident, $panic_name:ident, $number:ty, $method:ident, $len:expr, $value:expr) => {
        #[test]
        fn $ok_name() {
            let mut buf = $make_input(INPUT);

            let value = buf.$method($len);
            assert_eq!(buf.remaining(), 64 - $len);
            assert!(buf.has_remaining());
            assert_eq!(value, $value);
        }

        #[test]
        #[should_panic]
        fn $panic_name() {
            let mut buf = $make_input(&[]);

            let _ = buf.$method($len);
        }
    };
}

mod u8_slice {
  fn make_input(buf: &'static [u8]) -> &'static [u8] {
    buf
  }

  buf_tests!(make_input);
}

mod bytes {
  fn make_input(buf: &'static [u8]) -> impl Buf {
    Bytes::from_static(buf)
  }

  buf_tests!(make_input);
}

mod bytes_mut {
  fn make_input(buf: &'static [u8]) -> impl Buf {
    BytesMut::from(buf)
  }

  buf_tests!(make_input);
}

mod vec_deque {
  fn make_input(buf: &'static [u8]) -> impl Buf {
    let mut deque = VecDeque::new();

    if !buf.is_empty() {
      // Construct |b|some bytes|a| `VecDeque`
      let mid = buf.len() / 2;
      let (a, b) = buf.split_at(mid);

      deque.reserve_exact(buf.len() + 1);

      let extra_space = deque.capacity() - b.len() - 1;
      deque.resize(extra_space, 0);

      deque.extend(a);
      deque.drain(..extra_space);
      deque.extend(b);

      let (a, b) = deque.as_slices();
      assert!(
        !a.is_empty(),
        "could not setup test - attempt to create discontiguous VecDeque failed"
      );
      assert!(
        !b.is_empty(),
        "could not setup test - attempt to create discontiguous VecDeque failed"
      );
    }

    deque
  }

  buf_tests!(make_input, true);
}

#[cfg(feature = "std")]
mod cursor {
  use std::io::Cursor;

  fn make_input(buf: &'static [u8]) -> impl Buf {
    Cursor::new(buf)
  }

  buf_tests!(make_input);
}

mod box_bytes {
  fn make_input(buf: &'static [u8]) -> impl Buf {
    Box::new(Bytes::from_static(buf))
  }

  buf_tests!(make_input);
}

mod chain_u8_slice {
  fn make_input(buf: &'static [u8]) -> impl Buf {
    let (a, b) = buf.split_at(buf.len() / 2);
    Buf::chain(a, b)
  }

  buf_tests!(make_input);
}

mod chain_small_big_u8_slice {
  fn make_input(buf: &'static [u8]) -> impl Buf {
    let mid = cmp::min(1, buf.len());
    let (a, b) = buf.split_at(mid);
    Buf::chain(a, b)
  }

  buf_tests!(make_input);
}

mod chain_limited_slices {
  fn make_input(buf: &'static [u8]) -> impl Buf {
    let buf3 = &buf[cmp::min(buf.len(), 3)..];
    let a = Buf::take(buf3, 0);
    let b = Buf::take(buf, 3);
    let c = Buf::take(buf3, usize::MAX);
    let d = buf;
    Buf::take(Buf::chain(Buf::chain(a, b), Buf::chain(c, d)), buf.len())
  }

  buf_tests!(make_input, true);
}

// ---------------------------------------------------------------------------
// Forwarded `Buf` readers across every Buf impl. Calls go through the generic
// `B: Buf` bound, so each `get_*` / `try_get_*` resolves to the trait method
// (the macro-forwarded impl), not an inherent method.
// ---------------------------------------------------------------------------

fn check_reads<B: Buf>(make: impl Fn(&[u8]) -> B) {
  macro_rules! rd {
    ($be_bytes:expr, $le_bytes:expr, $get:ident, $get_le:ident, $get_ne:ident,
     $tg:ident, $tg_le:ident, $tg_ne:ident, $val:expr) => {{
      let val = $val;
      // Big-endian and little-endian byte patterns decode to the same value.
      assert_eq!(make(&$be_bytes).$get(), val);
      assert_eq!(make(&$le_bytes).$get_le(), val);
      let ne = make(if cfg!(target_endian = "big") {
        &$be_bytes
      } else {
        &$le_bytes
      })
      .$get_ne();
      assert_eq!(ne, val);
      // try_get variants: success then insufficient-data error.
      assert_eq!(make(&$be_bytes).$tg().unwrap(), val);
      assert_eq!(make(&$le_bytes).$tg_le().unwrap(), val);
      let tne = make(if cfg!(target_endian = "big") {
        &$be_bytes
      } else {
        &$le_bytes
      })
      .$tg_ne()
      .unwrap();
      assert_eq!(tne, val);
      assert!(make(&[]).$tg().is_err());
    }};
  }

  rd!(
    [0x01u8, 0x02],
    [0x02u8, 0x01],
    get_u16,
    get_u16_le,
    get_u16_ne,
    try_get_u16,
    try_get_u16_le,
    try_get_u16_ne,
    0x0102u16
  );
  rd!(
    [0xFFu8, 0xFE],
    [0xFEu8, 0xFF],
    get_i16,
    get_i16_le,
    get_i16_ne,
    try_get_i16,
    try_get_i16_le,
    try_get_i16_ne,
    -2i16
  );
  rd!(
    [0x01u8, 0x02, 0x03, 0x04],
    [0x04u8, 0x03, 0x02, 0x01],
    get_u32,
    get_u32_le,
    get_u32_ne,
    try_get_u32,
    try_get_u32_le,
    try_get_u32_ne,
    0x01020304u32
  );
  rd!(
    [0xFFu8, 0xFF, 0xFF, 0xFE],
    [0xFEu8, 0xFF, 0xFF, 0xFF],
    get_i32,
    get_i32_le,
    get_i32_ne,
    try_get_i32,
    try_get_i32_le,
    try_get_i32_ne,
    -2i32
  );
  rd!(
    [0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08],
    [0x08u8, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01],
    get_u64,
    get_u64_le,
    get_u64_ne,
    try_get_u64,
    try_get_u64_le,
    try_get_u64_ne,
    0x0102030405060708u64
  );
  rd!(
    [0xFFu8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE],
    [0xFEu8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF],
    get_i64,
    get_i64_le,
    get_i64_ne,
    try_get_i64,
    try_get_i64_le,
    try_get_i64_ne,
    -2i64
  );
  rd!(
    [
      0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
      0x10
    ],
    [
      0x10u8, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09, 0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02,
      0x01
    ],
    get_u128,
    get_u128_le,
    get_u128_ne,
    try_get_u128,
    try_get_u128_le,
    try_get_u128_ne,
    0x0102030405060708090A0B0C0D0E0F10u128
  );
  rd!(
    [
      0xFFu8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
      0xFE
    ],
    [
      0xFEu8, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
      0xFF
    ],
    get_i128,
    get_i128_le,
    get_i128_ne,
    try_get_i128,
    try_get_i128_le,
    try_get_i128_ne,
    -2i128
  );
  // 12.5 is exactly representable, so equality checks are safe.
  rd!(
    [0x41u8, 0x48, 0x00, 0x00],
    [0x00u8, 0x00, 0x48, 0x41],
    get_f32,
    get_f32_le,
    get_f32_ne,
    try_get_f32,
    try_get_f32_le,
    try_get_f32_ne,
    12.5f32
  );
  rd!(
    [0x40u8, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    [0x00u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40],
    get_f64,
    get_f64_le,
    get_f64_ne,
    try_get_f64,
    try_get_f64_le,
    try_get_f64_ne,
    12.5f64
  );

  // 8-bit readers.
  assert_eq!(make(&[0xAB]).get_u8(), 0xAB);
  assert_eq!(make(&[0xFB]).get_i8(), -5);
  assert_eq!(make(&[0xAB]).try_get_u8().unwrap(), 0xAB);
  assert_eq!(make(&[0xFB]).try_get_i8().unwrap(), -5);
  assert!(make(&[]).try_get_u8().is_err());
  assert!(make(&[]).try_get_i8().is_err());

  // Variable-width unsigned readers, nbytes 1..=8. The expected value is the
  // big-endian interpretation of the chosen bytes.
  let full = 0x1122334455667788u64;
  for nbytes in 1..=8usize {
    let be: Vec<u8> = full.to_be_bytes()[8 - nbytes..].to_vec();
    let v = be.iter().fold(0u64, |acc, &b| (acc << 8) | b as u64);
    let le: Vec<u8> = be.iter().rev().copied().collect();
    assert_eq!(make(&be).get_uint(nbytes), v);
    assert_eq!(make(&le).get_uint_le(nbytes), v);
    let ne = make(if cfg!(target_endian = "big") {
      &be
    } else {
      &le
    })
    .get_uint_ne(nbytes);
    assert_eq!(ne, v);
    assert_eq!(make(&be).try_get_uint(nbytes).unwrap(), v);
    assert_eq!(make(&le).try_get_uint_le(nbytes).unwrap(), v);
    assert!(make(&[]).try_get_uint(nbytes).is_err());
  }

  // Variable-width signed readers: all-ones sign-extends to -1.
  for nbytes in 1..=8usize {
    let ones = vec![0xFFu8; nbytes];
    assert_eq!(make(&ones).get_int(nbytes), -1);
    assert_eq!(make(&ones).get_int_le(nbytes), -1);
    let ne = make(&ones).get_int_ne(nbytes);
    assert_eq!(ne, -1);
    assert_eq!(make(&ones).try_get_int(nbytes).unwrap(), -1);
    assert_eq!(make(&ones).try_get_int_le(nbytes).unwrap(), -1);
    assert_eq!(make(&ones).try_get_int_ne(nbytes).unwrap(), -1);
    assert!(make(&[]).try_get_int(nbytes).is_err());
  }
}

#[test]
fn forwarded_buf_reads_buffer() {
  check_reads(|b: &[u8]| smol_bytes::Buffer::try_from(b).unwrap());
}

#[test]
fn forwarded_buf_reads_bytes_mut() {
  check_reads(|b: &[u8]| BytesMut::from(b));
}

#[test]
fn forwarded_buf_reads_shared_bytes() {
  check_reads(|b: &[u8]| smol_bytes::shared::Bytes::copy_from_slice(b));
}

#[test]
fn forwarded_buf_reads_compact_bytes() {
  check_reads(|b: &[u8]| smol_bytes::compact::Bytes::copy_from_slice(b));
}

#[allow(unused_allocation)] // This is intentional.
#[test]
fn test_deref_buf_forwards() {
  struct Special;

  impl Buf for Special {
    fn remaining(&self) -> usize {
      unreachable!("remaining");
    }

    fn chunk(&self) -> &[u8] {
      unreachable!("chunk");
    }

    fn advance(&mut self, _: usize) {
      unreachable!("advance");
    }

    fn get_u8(&mut self) -> u8 {
      // specialized!
      b'x'
    }
  }

  // these should all use the specialized method
  assert_eq!(Special.get_u8(), b'x');
  assert_eq!((&mut Special as &mut dyn Buf).get_u8(), b'x');
  assert_eq!((Box::new(Special) as Box<dyn Buf>).get_u8(), b'x');
  assert_eq!(Box::new(Special).get_u8(), b'x');
}
