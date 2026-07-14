#![warn(rust_2018_idioms)]

use smol_bytes::{Buf, BufMut, Bytes, BytesMut, INLINE_CAP};
use std::cmp::Ordering as CmpOrdering;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Bound;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use std::panic::{self, AssertUnwindSafe};

const LONG: &[u8] = b"mary had a little lamb, little lamb, little lamb...............";
const SHORT: &[u8] = b"hello world";

fn is_sync<T: Sync>() {}
fn is_send<T: Send>() {}

#[test]
fn test_bounds() {
  is_sync::<Bytes>();
  is_sync::<BytesMut>();
  is_send::<Bytes>();
  is_send::<BytesMut>();
}

#[test]
fn test_layout() {
  use std::mem;

  assert_eq!(
    mem::size_of::<Bytes>(),
    INLINE_CAP + 2,
    "Bytes size should be the inline capacity plus two metadata bytes",
  );
  assert_eq!(
    mem::size_of::<BytesMut>(),
    INLINE_CAP + 2,
    "BytesMut size should be the inline capacity plus two metadata bytes",
  );

  assert_eq!(
    mem::size_of::<Bytes>(),
    mem::size_of::<Option<Bytes>>(),
    "Bytes should be same size as Option<Bytes>",
  );

  assert_eq!(
    mem::size_of::<BytesMut>(),
    mem::size_of::<Option<BytesMut>>(),
    "BytesMut should be same size as Option<BytesMut>",
  );
}

#[test]
fn from_slice() {
  let a = Bytes::from(&b"abcdefgh"[..]);
  assert_eq!(a, b"abcdefgh"[..]);
  assert_eq!(a, &b"abcdefgh"[..]);
  assert_eq!(a, Vec::from(&b"abcdefgh"[..]));
  assert_eq!(b"abcdefgh"[..], a);
  assert_eq!(&b"abcdefgh"[..], a);
  assert_eq!(Vec::from(&b"abcdefgh"[..]), a);

  let a = BytesMut::from(&b"abcdefgh"[..]);
  assert_eq!(a, b"abcdefgh"[..]);
  assert_eq!(a, &b"abcdefgh"[..]);
  assert_eq!(a, Vec::from(&b"abcdefgh"[..]));
  assert_eq!(b"abcdefgh"[..], a);
  assert_eq!(&b"abcdefgh"[..], a);
  assert_eq!(Vec::from(&b"abcdefgh"[..]), a);
}

fn heap_bytes(input: &[u8]) -> Bytes {
  let mut bytes = BytesMut::from(input);
  bytes.make_heap();
  assert!(!bytes.is_inline());
  let bytes = bytes.freeze_shared();
  assert!(!bytes.is_inline());
  bytes
}

fn heap_bytes_mut(input: &[u8]) -> BytesMut {
  let mut bytes = BytesMut::from(input);
  bytes.make_heap();
  assert!(!bytes.is_inline());
  bytes
}

#[test]
fn heap_signed_variable_width_reads_match_infallible_reads() {
  const BIG_ENDIAN: [u8; 3] = [0xff, 0x00, 0x01];
  const LITTLE_ENDIAN: [u8; 3] = [0x01, 0x00, 0xff];

  macro_rules! assert_signed_read_parity {
    ($constructor:ident, $input:expr, $get:ident, $try_get:ident) => {{
      let mut infallible = $constructor($input);
      let mut fallible = $constructor($input);

      assert!(!infallible.is_inline());
      assert!(!fallible.is_inline());
      assert_eq!(infallible.$get(3), -65_535);
      assert_eq!(fallible.$try_get(3), Ok(-65_535));
      assert_eq!(infallible.remaining(), 0);
      assert_eq!(fallible.remaining(), 0);
    }};
  }

  let native_endian = if cfg!(target_endian = "big") {
    BIG_ENDIAN
  } else {
    LITTLE_ENDIAN
  };

  assert_signed_read_parity!(heap_bytes, &BIG_ENDIAN, get_int, try_get_int);
  assert_signed_read_parity!(heap_bytes, &LITTLE_ENDIAN, get_int_le, try_get_int_le);
  assert_signed_read_parity!(heap_bytes, &native_endian, get_int_ne, try_get_int_ne);

  assert_signed_read_parity!(heap_bytes_mut, &BIG_ENDIAN, get_int, try_get_int);
  assert_signed_read_parity!(heap_bytes_mut, &LITTLE_ENDIAN, get_int_le, try_get_int_le);
  assert_signed_read_parity!(heap_bytes_mut, &native_endian, get_int_ne, try_get_int_ne);
}

#[test]
fn heap_signed_variable_width_edge_reads_match_infallible_reads() {
  const EMPTY: [u8; 0] = [];
  const NEGATIVE: [u8; 1] = [0x80];
  const BIG_ENDIAN_BOUNDARY: [u8; 8] = [0x80, 0, 0, 0, 0, 0, 0, 0];
  const LITTLE_ENDIAN_BOUNDARY: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0x80];

  macro_rules! assert_zero_width_read {
    ($constructor:ident, $get:ident, $try_get:ident) => {{
      let mut infallible = $constructor(&EMPTY);
      let mut fallible = $constructor(&EMPTY);
      let infallible_remaining = infallible.remaining();
      let fallible_remaining = fallible.remaining();

      assert!(!infallible.is_inline());
      assert!(!fallible.is_inline());
      assert_eq!(infallible.$get(0), 0);
      assert_eq!(fallible.$try_get(0), Ok(0));
      assert_eq!(infallible.remaining(), infallible_remaining);
      assert_eq!(fallible.remaining(), fallible_remaining);
    }};
  }

  macro_rules! assert_nonzero_width_read {
    ($constructor:ident, $input:expr, $nbytes:expr, $expected:expr, $get:ident, $try_get:ident) => {{
      let mut infallible = $constructor($input);
      let mut fallible = $constructor($input);
      let infallible_remaining = infallible.remaining();
      let fallible_remaining = fallible.remaining();

      assert!(!infallible.is_inline());
      assert!(!fallible.is_inline());
      assert_eq!(infallible.$get($nbytes), $expected);
      assert_eq!(fallible.$try_get($nbytes), Ok($expected));
      assert_eq!(infallible.remaining(), infallible_remaining - $nbytes);
      assert_eq!(fallible.remaining(), fallible_remaining - $nbytes);
    }};
  }

  let native_endian_boundary = if cfg!(target_endian = "big") {
    BIG_ENDIAN_BOUNDARY
  } else {
    LITTLE_ENDIAN_BOUNDARY
  };

  assert_zero_width_read!(heap_bytes, get_int, try_get_int);
  assert_zero_width_read!(heap_bytes, get_int_le, try_get_int_le);
  assert_zero_width_read!(heap_bytes, get_int_ne, try_get_int_ne);
  assert_nonzero_width_read!(heap_bytes, &NEGATIVE, 1, -128, get_int, try_get_int);
  assert_nonzero_width_read!(heap_bytes, &NEGATIVE, 1, -128, get_int_le, try_get_int_le);
  assert_nonzero_width_read!(heap_bytes, &NEGATIVE, 1, -128, get_int_ne, try_get_int_ne);
  assert_nonzero_width_read!(
    heap_bytes,
    &BIG_ENDIAN_BOUNDARY,
    8,
    i64::MIN,
    get_int,
    try_get_int
  );
  assert_nonzero_width_read!(
    heap_bytes,
    &LITTLE_ENDIAN_BOUNDARY,
    8,
    i64::MIN,
    get_int_le,
    try_get_int_le
  );
  assert_nonzero_width_read!(
    heap_bytes,
    &native_endian_boundary,
    8,
    i64::MIN,
    get_int_ne,
    try_get_int_ne
  );

  assert_zero_width_read!(heap_bytes_mut, get_int, try_get_int);
  assert_zero_width_read!(heap_bytes_mut, get_int_le, try_get_int_le);
  assert_zero_width_read!(heap_bytes_mut, get_int_ne, try_get_int_ne);
  assert_nonzero_width_read!(heap_bytes_mut, &NEGATIVE, 1, -128, get_int, try_get_int);
  assert_nonzero_width_read!(
    heap_bytes_mut,
    &NEGATIVE,
    1,
    -128,
    get_int_le,
    try_get_int_le
  );
  assert_nonzero_width_read!(
    heap_bytes_mut,
    &NEGATIVE,
    1,
    -128,
    get_int_ne,
    try_get_int_ne
  );
  assert_nonzero_width_read!(
    heap_bytes_mut,
    &BIG_ENDIAN_BOUNDARY,
    8,
    i64::MIN,
    get_int,
    try_get_int
  );
  assert_nonzero_width_read!(
    heap_bytes_mut,
    &LITTLE_ENDIAN_BOUNDARY,
    8,
    i64::MIN,
    get_int_le,
    try_get_int_le
  );
  assert_nonzero_width_read!(
    heap_bytes_mut,
    &native_endian_boundary,
    8,
    i64::MIN,
    get_int_ne,
    try_get_int_ne
  );
}

#[test]
fn fmt() {
  let a = format!("{:?}", Bytes::from(&b"abcdefg"[..]));
  let b = "b\"abcdefg\"";

  assert_eq!(a, b);

  let a = format!("{:?}", BytesMut::from(&b"abcdefg"[..]));
  assert_eq!(a, b);
}

#[test]
fn fmt_write() {
  use std::fmt::Write;
  let s = String::from_iter((0..10).map(|_| "abcdefg"));

  let mut a = BytesMut::with_capacity(64);
  write!(a, "{}", &s[..64]).unwrap();
  assert_eq!(a, &s.as_bytes()[..64]);

  let mut b = BytesMut::with_capacity(64);
  write!(b, "{}", &s[..32]).unwrap();
  write!(b, "{}", &s[32..64]).unwrap();
  assert_eq!(b, &s.as_bytes()[..64]);

  let mut c = BytesMut::with_capacity(64);
  write!(c, "{}", s).unwrap();
  assert_eq!(c, s.as_bytes());
}

#[test]
fn len() {
  let a = Bytes::from(&b"abcdefg"[..]);
  assert_eq!(a.len(), 7);

  let a = BytesMut::from(&b"abcdefg"[..]);
  assert_eq!(a.len(), 7);

  let a = Bytes::from(&b""[..]);
  assert!(a.is_empty());

  let a = BytesMut::from(&b""[..]);
  assert!(a.is_empty());
}

#[test]
fn index() {
  let a = Bytes::from(&b"hello world"[..]);
  assert_eq!(a[0..5], *b"hello");
}

#[test]
fn slice() {
  let a = Bytes::from(&b"hello world"[..]);

  let b = a.slice(3..5);
  assert_eq!(b, b"lo"[..]);

  let b = a.slice(0..0);
  assert_eq!(b, b""[..]);

  let b = a.slice(3..3);
  assert_eq!(b, b""[..]);

  let b = a.slice(a.len()..a.len());
  assert_eq!(b, b""[..]);

  let b = a.slice(..5);
  assert_eq!(b, b"hello"[..]);

  let b = a.slice(3..);
  assert_eq!(b, b"lo world"[..]);
}

#[test]
#[should_panic]
fn slice_oob_1() {
  let a = Bytes::from(&b"hello world"[..]);
  a.slice(5..44);
}

#[test]
#[should_panic]
fn slice_oob_2() {
  let a = Bytes::from(&b"hello world"[..]);
  a.slice(44..49);
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
#[should_panic]
fn slice_start_greater_than_end() {
  let a = Bytes::from(&b"hello world"[..]);
  a.slice(5..3);
}

#[test]
fn split_off() {
  let mut hello = Bytes::from(&b"helloworld"[..]);
  let world = hello.split_off(5);

  assert_eq!(hello, &b"hello"[..]);
  assert_eq!(world, &b"world"[..]);

  let mut hello = BytesMut::from(&b"helloworld"[..]);
  match hello.split_off(5) {
    Ok(_) => panic!("Expected Err for inline buffer"),
    Err(world) => {
      assert_eq!(hello, &b"hello"[..]);
      assert_eq!(world, &b"world"[..]);
    }
  }

  let mut hello = BytesMut::from(&b"helloworld"[..]);
  hello.make_heap();
  let world = hello.split_off(5).unwrap();
  assert_eq!(hello, &b"hello"[..]);
  assert_eq!(world, &b"world"[..]);
}

#[test]
#[should_panic]
fn split_off_oob() {
  let mut hello = Bytes::from(&b"helloworld"[..]);
  let _ = hello.split_off(44);
}

#[test]
#[should_panic = "split_off out of bounds"]
fn bytes_mut_split_off_oob() {
  let mut hello = BytesMut::from(LONG);
  let _ = hello.split_off(444);
}

#[test]
fn split_off_uninitialized() {
  let mut bytes = BytesMut::with_capacity(1024);
  let other = bytes.split_off(128).unwrap();

  assert_eq!(bytes.len(), 0);
  assert_eq!(bytes.capacity(), 128);

  assert_eq!(other.len(), 0);
  assert_eq!(other.capacity(), 896);
}

#[test]
fn split_off_uninitialized_inline_promotes_to_heap() {
  // `new()` creates an inline buffer with len=0 and INLINE_CAP capacity.
  // Splitting at 30 exceeds written len but is within capacity — the
  // implementation must promote to heap and partition the capacity.
  let mut bytes = BytesMut::new();
  assert!(bytes.is_inline());
  let other = bytes.split_off(30).unwrap();

  assert_eq!(bytes.len(), 0);
  assert!(bytes.capacity() >= 30);
  assert_eq!(other.len(), 0);
  assert!(other.capacity() >= INLINE_CAP - 30);
}

#[test]
fn split_off_at_capacity_returns_empty_tail() {
  let mut bytes = BytesMut::with_capacity(1024);
  let cap = bytes.capacity();
  let other = bytes.split_off(cap).unwrap();
  assert_eq!(bytes.capacity(), cap);
  assert_eq!(other.capacity(), 0);
  assert_eq!(other.len(), 0);
}

#[test]
#[should_panic = "split_off out of bounds"]
fn split_off_beyond_capacity_panics() {
  let mut bytes = BytesMut::with_capacity(1024);
  let _ = bytes.split_off(2000);
}

#[test]
fn split_off_preserves_written_data() {
  let mut bytes = BytesMut::with_capacity(1024);
  bytes.extend_from_slice(b"hello");
  let other = bytes.split_off(10).unwrap();
  assert_eq!(bytes.len(), 5);
  assert_eq!(bytes.as_slice(), b"hello");
  assert_eq!(bytes.capacity(), 10);
  assert_eq!(other.len(), 0);
  assert_eq!(other.capacity(), 1014);
}

#[test]
fn split_off_to_loop() {
  let s = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

  for i in 0..(s.len() + 1) {
    {
      let mut bytes = Bytes::from(&s[..]);
      let off = bytes.split_off(i);
      assert_eq!(i, bytes.len());
      let mut sum: Vec<u8> = Vec::new();
      sum.extend(bytes.iter());
      sum.extend(off.iter());
      assert_eq!(&s[..], &sum[..]);
    }
    {
      let mut bytes = BytesMut::from(&s[..]);
      match bytes.split_off(i) {
        Ok(off) => {
          // Heap buffer - got BytesMut (can grow)
          assert_eq!(i, bytes.len());
          let mut sum: Vec<u8> = Vec::new();
          sum.extend(&bytes);
          sum.extend(&off);
          assert_eq!(&s[..], &sum[..]);
        }
        Err(off) => {
          // Inline buffer - got Buffer (max 62 bytes), bytes was truncated
          assert_eq!(i, bytes.len());
          let mut sum: Vec<u8> = Vec::new();
          sum.extend(&bytes);
          sum.extend(&off);
          assert_eq!(&s[..], &sum[..]);
        }
      }
    }
    {
      let mut bytes = Bytes::from(&s[..]);
      let off = bytes.split_to(i);
      assert_eq!(i, off.len());
      let mut sum: Vec<u8> = Vec::new();
      sum.extend(off.iter());
      sum.extend(bytes.iter());
      assert_eq!(&s[..], &sum[..]);
    }
    {
      let mut bytes = BytesMut::from(&s[..]);
      match bytes.split_to(i) {
        Ok(off) => {
          // Heap buffer - got BytesMut (can grow)
          assert_eq!(i, off.len());
          let mut sum: Vec<u8> = Vec::new();
          sum.extend(&off);
          sum.extend(&bytes);
          assert_eq!(&s[..], &sum[..]);
        }
        Err(off) => {
          // Inline buffer - got Buffer (max 62 bytes), bytes was advanced
          assert_eq!(i, off.len());
          let mut sum: Vec<u8> = Vec::new();
          sum.extend(&off);
          sum.extend(&bytes);
          assert_eq!(&s[..], &sum[..]);
        }
      }
    }
  }
}

#[test]
fn split_to_1() {
  // Static
  let mut a = Bytes::from_static(SHORT);
  let b = a.split_to(4);

  assert_eq!(SHORT[4..], a);
  assert_eq!(SHORT[..4], b);

  // Allocated
  let mut a = Bytes::copy_from_slice(LONG);
  let b = a.split_to(4);

  assert_eq!(LONG[4..], a);
  assert_eq!(LONG[..4], b);

  let mut a = Bytes::copy_from_slice(LONG);
  let b = a.split_to(30);

  assert_eq!(LONG[30..], a);
  assert_eq!(LONG[..30], b);
}

#[test]
fn split_to_2() {
  let mut a = Bytes::from(LONG);
  assert_eq!(LONG, a);

  let b = a.split_to(1);

  assert_eq!(LONG[1..], a);
  drop(b);
}

#[test]
#[should_panic]
fn split_to_oob() {
  let mut hello = Bytes::from(&b"helloworld"[..]);
  let _ = hello.split_to(33);
}

#[test]
#[should_panic]
fn split_to_oob_mut() {
  let mut hello = BytesMut::from(LONG);
  let _ = hello.split_to(333);
}

#[test]
#[should_panic]
fn split_to_uninitialized() {
  let mut bytes = BytesMut::with_capacity(1024);
  let _other = bytes.split_to(128);
}

#[test]
#[cfg_attr(not(panic = "unwind"), ignore)]
fn split_off_to_at_gt_len() {
  fn make_bytes() -> Bytes {
    let mut bytes = BytesMut::with_capacity(100);
    bytes.put_slice(&[10, 20, 30, 40]);
    bytes.freeze_shared()
  }

  use std::panic;

  let _ = make_bytes().split_to(4);
  let _ = make_bytes().split_off(4);

  assert!(panic::catch_unwind(move || {
    let _ = make_bytes().split_to(5);
  })
  .is_err());

  assert!(panic::catch_unwind(move || {
    let _ = make_bytes().split_off(5);
  })
  .is_err());
}

#[test]
fn truncate() {
  let s = &b"helloworld"[..];
  let mut hello = Bytes::from(s);
  hello.truncate(15);
  assert_eq!(hello, s);
  hello.truncate(10);
  assert_eq!(hello, s);
  hello.truncate(5);
  assert_eq!(hello, "hello");
}

#[test]
#[allow(clippy::reversed_empty_ranges)]
fn fallible_slicing_is_checked_for_inline_and_shared_heap() {
  let mut inline = Bytes::from_static(b"abcdef");
  inline.advance(2);
  assert_eq!(inline.try_slice(0..4).unwrap(), b"cdef"[..]);
  assert!(inline.try_slice(4..5).is_err());
  assert!(inline.try_split_to(5).is_err());

  let heap = Bytes::from(vec![b'x'; 100]);
  assert!(heap.is_heap());
  assert!(heap.try_slice(80..20).is_err());
  assert!(heap.try_slice(..=usize::MAX).is_err());
  assert!(heap
    .try_slice((Bound::Excluded(usize::MAX), Bound::Unbounded))
    .is_err());
}

#[test]
fn same_heap_pointer_with_different_lengths_obeys_eq_hash_ord_laws() {
  fn hash(bytes: &Bytes) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
  }

  let full = Bytes::from(vec![b'x'; 100]);
  let prefix = full.slice(..99);
  assert!(full.is_heap());
  assert!(prefix.is_heap());
  assert_eq!(full.as_ptr(), prefix.as_ptr());

  assert_ne!(full, prefix);
  assert_ne!(hash(&full), hash(&prefix));
  assert_eq!(full.cmp(&prefix), CmpOrdering::Greater);
  assert_eq!(full.partial_cmp(&prefix), Some(CmpOrdering::Greater));
}

#[test]
fn freeze_clone_shared() {
  let s = &b"abcdefgh"[..];
  let mut b = BytesMut::from(s);
  b.make_heap();
  let b = b.split().unwrap().freeze_shared();
  assert_eq!(b, s);
  let c = b.clone();
  assert_eq!(c, s);
}

#[test]
fn freeze_shared_clone_unique() {
  let s = &b"abcdefgh"[..];
  let b = BytesMut::from(s).freeze_shared();
  assert_eq!(b, s);
  let c = b.clone();
  assert_eq!(c, s);
}

#[test]
fn freeze_shared_after_advance() {
  let s = &b"abcdefgh"[..];
  let mut b = BytesMut::from(s);
  b.advance(1);
  assert_eq!(b, s[1..]);
  let b = b.freeze_shared();
  // Verify fix for #352. Previously, freeze_shared would ignore the start offset
  // for BytesMuts in Vec mode.
  assert_eq!(b, s[1..]);
}

#[test]
fn freeze_shared_after_advance_arc() {
  let s = &b"abcdefgh"[..];
  let mut b = BytesMut::from(s);
  // Make b Arc
  let _ = b.split_to(0);
  b.advance(1);
  assert_eq!(b, s[1..]);
  let b = b.freeze_shared();
  assert_eq!(b, s[1..]);
}

#[test]
fn freeze_shared_after_split_to() {
  let s = &b"abcdefgh"[..];
  let mut b = BytesMut::from(s);
  b.make_heap();
  let _ = b.split_to(1);
  assert_eq!(b, s[1..]);
  let b = b.freeze_shared();
  assert_eq!(b, s[1..]);
}

#[test]
fn freeze_shared_after_truncate() {
  let s = &b"abcdefgh"[..];
  let mut b = BytesMut::from(s);
  b.truncate(7);
  assert_eq!(b, s[..7]);
  let b = b.freeze_shared();
  assert_eq!(b, s[..7]);
}

#[test]
fn freeze_shared_after_truncate_arc() {
  let s = &b"abcdefgh"[..];
  let mut b = BytesMut::from(s);
  // Make b Arc
  let _ = b.split_to(0);
  b.truncate(7);
  assert_eq!(b, s[..7]);
  let b = b.freeze_shared();
  assert_eq!(b, s[..7]);
}

#[test]
fn freeze_shared_after_split_off() {
  let s = &b"abcdefgh"[..];
  let mut b = BytesMut::from(s);
  b.make_heap();
  let _ = b.split_off(7);
  assert_eq!(b, s[..7]);
  let b = b.freeze_shared();
  assert_eq!(b, s[..7]);
}

#[test]
#[allow(warnings)]
fn fns_defined_for_bytes_mut() {
  let mut bytes = BytesMut::from(&b"hello world"[..]);

  let _ = bytes.as_ptr();
  let _ = bytes.as_mut_ptr();

  // Iterator
  let v: Vec<u8> = bytes.as_ref().iter().cloned().collect();
  assert_eq!(&v[..], bytes);
}

#[test]
fn reserve_convert() {
  // Vec -> Vec
  let mut bytes = BytesMut::from(LONG);
  bytes.reserve(64);
  assert_eq!(bytes.capacity(), LONG.len() + 64);

  // Arc -> Vec
  let mut bytes = BytesMut::from(LONG);
  let a = bytes.split_to(30);

  bytes.reserve(128);
  assert!(bytes.capacity() >= bytes.len() + 128);

  drop(a);
}

#[test]
fn reserve_growth() {
  let mut bytes = BytesMut::with_capacity(64);
  bytes.put("hello world".as_bytes());
  let _ = bytes.split();

  bytes.reserve(65);
  assert_eq!(bytes.capacity(), 117);
}

#[test]
fn reserve_allocates_at_least_original_capacity() {
  let mut bytes = BytesMut::with_capacity(1024);

  for i in 0..1020 {
    bytes.put_u8(i as u8);
  }

  let _other = bytes.split();

  bytes.reserve(16);
  assert_eq!(bytes.capacity(), 1024);
}

#[test]
#[cfg_attr(miri, ignore)] // Miri is too slow
fn reserve_max_original_capacity_value() {
  const SIZE: usize = 128 * 1024;

  let mut bytes = BytesMut::with_capacity(SIZE);

  for _ in 0..SIZE {
    bytes.put_u8(0u8);
  }

  let _other = bytes.split();

  bytes.reserve(16);
  assert_eq!(bytes.capacity(), 64 * 1024);
}

#[test]
fn reserve_in_arc_unique_does_not_overallocate() {
  let mut bytes = BytesMut::with_capacity(1000);
  let _ = bytes.split();

  // now bytes is Arc and refcount == 1

  assert_eq!(1000, bytes.capacity());
  bytes.reserve(2001);
  assert_eq!(2001, bytes.capacity());
}

#[test]
fn reserve_in_arc_unique_doubles() {
  let mut bytes = BytesMut::with_capacity(1000);
  let _ = bytes.split();

  // now bytes is Arc and refcount == 1

  assert_eq!(1000, bytes.capacity());
  bytes.reserve(1001);
  assert_eq!(2000, bytes.capacity());
}

#[test]
fn reserve_in_arc_unique_does_not_overallocate_after_split() {
  let mut bytes = BytesMut::from(LONG);
  let orig_capacity = bytes.capacity();
  drop(bytes.split_off(LONG.len() / 2));

  // now bytes is Arc and refcount == 1

  let new_capacity = bytes.capacity();
  bytes.reserve(orig_capacity - new_capacity);
  assert_eq!(bytes.capacity(), orig_capacity);
}

#[test]
fn reserve_in_arc_unique_does_not_overallocate_after_multiple_splits() {
  let mut bytes = BytesMut::from(LONG);
  let orig_capacity = bytes.capacity();
  for _ in 0..10 {
    drop(bytes.split_off(LONG.len() / 2));

    // now bytes is Arc and refcount == 1

    let new_capacity = bytes.capacity();
    bytes.reserve(orig_capacity - new_capacity);
  }
  assert_eq!(bytes.capacity(), orig_capacity);
}

#[test]
fn reserve_in_arc_nonunique_does_not_overallocate() {
  let mut bytes = BytesMut::with_capacity(1000);
  let _copy = bytes.split();

  // now bytes is Arc and refcount == 2

  assert_eq!(1000, bytes.capacity());
  bytes.reserve(2001);
  assert_eq!(2001, bytes.capacity());
}

/// This function tests `BytesMut::reserve_inner`, where `BytesMut` holds
/// a unique reference to the shared vector and decide to reuse it
/// by reallocating the `Vec`.
#[test]
fn reserve_shared_reuse() {
  let mut bytes = BytesMut::with_capacity(1000);
  bytes.put_slice(b"Hello, World!");
  drop(bytes.split());

  bytes.put_slice(b"!123ex123,sadchELLO,_wORLD!");
  // Use split_off so that v.capacity() - self.cap != off
  drop(bytes.split_off(9));
  assert_eq!(&*bytes, b"!123ex123");

  bytes.reserve(2000);
  assert_eq!(&*bytes, b"!123ex123");
  assert_eq!(bytes.capacity(), 2009);
}

#[test]
fn extend_mut() {
  let mut bytes = BytesMut::with_capacity(0);
  bytes.extend(LONG);
  assert_eq!(*bytes, LONG[..]);
}

#[test]
fn extend_from_slice_mut() {
  for &i in &[3, 34] {
    let mut bytes = BytesMut::new();
    bytes.extend_from_slice(&LONG[..i]);
    bytes.extend_from_slice(&LONG[i..]);
    assert_eq!(LONG[..], *bytes);
  }
}

#[test]
fn extend_mut_from_bytes() {
  let mut bytes = BytesMut::with_capacity(0);
  bytes.extend([Bytes::from(LONG)]);
  assert_eq!(*bytes, LONG[..]);
}

#[test]
fn extend_past_lower_limit_of_size_hint() {
  // See https://github.com/tokio-rs/bytes/pull/674#pullrequestreview-1913035700
  struct Iter<I>(I);

  impl<I: Iterator<Item = u8>> Iterator for Iter<I> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
      self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
      (5, None)
    }
  }

  let mut bytes = BytesMut::with_capacity(5);
  bytes.extend(Iter(std::iter::repeat_n(0, 10)));
  assert_eq!(bytes.len(), 10);
}

#[test]
fn extend_mut_without_size_hint() {
  let mut bytes = BytesMut::with_capacity(0);
  let mut long_iter = LONG.iter();

  // Use iter::from_fn since it doesn't know a size_hint
  bytes.extend(std::iter::from_fn(|| long_iter.next()));
  assert_eq!(*bytes, LONG[..]);
}

#[test]
fn from_static() {
  let mut a = Bytes::from_static(b"ab");
  let b = a.split_off(1);

  assert_eq!(a, b"a"[..]);
  assert_eq!(b, b"b"[..]);
}

#[test]
fn advance_static() {
  let mut a = Bytes::from_static(b"hello world");
  a.advance(6);
  assert_eq!(a, &b"world"[..]);
}

#[test]
fn advance_vec() {
  let mut a = Bytes::from(b"hello world boooo yah world zomg wat wat".to_vec());
  a.advance(16);
  assert_eq!(a, b"o yah world zomg wat wat"[..]);

  a.advance(4);
  assert_eq!(a, b"h world zomg wat wat"[..]);

  a.advance(6);
  assert_eq!(a, b"d zomg wat wat"[..]);
}

#[test]
fn advance_bytes_mut() {
  let mut a = BytesMut::from("hello world boooo yah world zomg wat wat");
  a.advance(16);
  assert_eq!(a, b"o yah world zomg wat wat"[..]);

  a.advance(4);
  assert_eq!(a, b"h world zomg wat wat"[..]);

  // Reserve some space.
  a.reserve(1024);
  assert_eq!(a, b"h world zomg wat wat"[..]);

  a.advance(6);
  assert_eq!(a, b"d zomg wat wat"[..]);
}

// Ensures BytesMut::advance reduces always capacity
//
// See https://github.com/tokio-rs/bytes/issues/725
#[test]
fn advance_bytes_mut_remaining_capacity() {
  // reduce the search space under miri
  let max_capacity = if cfg!(miri) { 64 } else { 256 };
  for capacity in 0..=max_capacity {
    for len in 0..=capacity {
      for advance in 0..=len {
        eprintln!("testing capacity={capacity}, len={len}, advance={advance}");
        let mut buf = BytesMut::with_capacity(capacity);

        buf.resize(len, 42);
        assert_eq!(buf.len(), len, "resize should write `len` bytes");
        assert_eq!(
          buf.remaining(),
          len,
          "Buf::remaining() should equal BytesMut::len"
        );

        buf.advance(advance);
        assert_eq!(
          buf.remaining(),
          len - advance,
          "Buf::advance should reduce the remaining len"
        );

        if capacity <= INLINE_CAP {
          assert_eq!(
            buf.capacity(),
            INLINE_CAP - advance,
            "Buf::advance should reduce the remaining capacity"
          );
        } else {
          assert_eq!(
            buf.capacity(),
            capacity - advance,
            "Buf::advance should reduce the remaining capacity"
          );
        }
      }
    }
  }
}

#[test]
#[should_panic]
fn advance_past_len() {
  let mut a = BytesMut::from("hello world");
  a.advance(20);
}

#[test]
#[should_panic]
fn mut_advance_past_len() {
  let mut a = BytesMut::from("hello world");
  unsafe {
    a.advance_mut(100);
  }
}

#[test]
// Only run these tests on little endian systems. CI uses qemu for testing
// big endian... and qemu doesn't really support threading all that well.
#[cfg(any(miri, target_endian = "little"))]
#[cfg(not(target_family = "wasm"))] // wasm without experimental threads proposal doesn't support threads
fn stress() {
  // Tests promoting a buffer from a vec -> shared in a concurrent situation
  use std::sync::{Arc, Barrier};
  use std::thread;

  const THREADS: usize = 8;
  const ITERS: usize = if cfg!(miri) { 100 } else { 1_000 };

  for i in 0..ITERS {
    let data = [i as u8; 256];
    let buf = Arc::new(Bytes::copy_from_slice(&data[..]));

    let barrier = Arc::new(Barrier::new(THREADS));
    let mut joins = Vec::with_capacity(THREADS);

    for _ in 0..THREADS {
      let c = barrier.clone();
      let buf = buf.clone();

      joins.push(thread::spawn(move || {
        c.wait();
        let buf: Bytes = (*buf).clone();
        drop(buf);
      }));
    }

    for th in joins {
      th.join().unwrap();
    }

    assert_eq!(*buf, data[..]);
  }
}

#[test]
fn partial_eq_bytesmut() {
  let bytes = Bytes::from(&b"The quick red fox"[..]);
  let bytesmut = BytesMut::from(&b"The quick red fox"[..]);
  assert!(bytes == bytesmut);
  assert!(bytesmut == bytes);
  let bytes2 = Bytes::from(&b"Jumped over the lazy brown dog"[..]);
  assert!(bytes2 != bytesmut);
  assert!(bytesmut != bytes2);
}

#[test]
fn bytes_mut_unsplit_basic() {
  let mut buf = BytesMut::with_capacity(64);
  buf.extend_from_slice(b"aaabbbcccddd");

  buf.make_heap();
  let splitted = buf.split_off(6).unwrap();
  assert_eq!(b"aaabbb", &buf[..]);
  assert_eq!(b"cccddd", &splitted[..]);

  buf.unsplit(splitted);
  assert_eq!(b"aaabbbcccddd", &buf[..]);
}

#[test]
fn bytes_mut_unsplit_empty_other() {
  let mut buf = BytesMut::with_capacity(64);
  buf.extend_from_slice(b"aaabbbcccddd");

  // empty other
  let other = BytesMut::new();

  buf.unsplit(other);
  assert_eq!(b"aaabbbcccddd", &buf[..]);
}

#[test]
fn bytes_mut_unsplit_empty_self() {
  // empty self
  let mut buf = BytesMut::new();
  let mut other = BytesMut::with_capacity(64);
  other.extend_from_slice(b"aaabbbcccddd");
  buf.make_heap();
  other.make_heap();
  buf.unsplit(other);
  assert_eq!(b"aaabbbcccddd", &buf[..]);
}

#[test]
fn bytes_mut_unsplit_other_keeps_capacity() {
  let mut buf = BytesMut::with_capacity(64);
  buf.extend_from_slice(b"aabb");

  // non empty other created "from" buf
  buf.make_heap();
  let mut other = buf.split_off(buf.len()).unwrap();
  other.extend_from_slice(b"ccddee");
  buf.unsplit(other);

  assert_eq!(buf.capacity(), 64);
}

#[test]
fn bytes_mut_unsplit_empty_other_keeps_capacity() {
  let mut buf = BytesMut::with_capacity(64);
  buf.extend_from_slice(b"aabbccddee");

  // empty other created "from" buf
  buf.make_heap();
  let other = buf.split_off(buf.len()).unwrap();
  buf.unsplit(other);

  assert_eq!(buf.capacity(), 64);
}

#[test]
fn bytes_mut_unsplit_arc_different() {
  let mut buf = BytesMut::with_capacity(64);
  buf.extend_from_slice(b"aaaabbbbeeee");
  buf.make_heap();
  let _ = buf.split_off(8); //arc

  let mut buf2 = BytesMut::with_capacity(64);
  buf2.extend_from_slice(b"ccccddddeeee");

  let _ = buf2.split_off(8); //arc

  buf.unsplit(buf2);
  assert_eq!(b"aaaabbbbccccdddd", &buf[..]);
}

#[test]
fn bytes_mut_unsplit_arc_non_contiguous() {
  let mut buf = BytesMut::with_capacity(64);
  buf.extend_from_slice(b"aaaabbbbeeeeccccdddd");

  buf.make_heap();
  let mut buf2 = buf.split_off(8).unwrap(); //arc

  let buf3 = buf2.split_off(4).unwrap(); //arc

  buf.unsplit(buf3);
  assert_eq!(b"aaaabbbbccccdddd", &buf[..]);
}

#[test]
fn bytes_mut_unsplit_two_split_offs() {
  let mut buf = BytesMut::with_capacity(64);
  buf.extend_from_slice(b"aaaabbbbccccdddd");

  buf.make_heap();
  let mut buf2 = buf.split_off(8).unwrap(); //arc
  let buf3 = buf2.split_off(4).unwrap(); //arc

  buf2.unsplit(buf3);
  buf.unsplit(buf2);
  assert_eq!(b"aaaabbbbccccdddd", &buf[..]);
}

#[test]
fn from_iter_no_size_hint() {
  use std::iter;

  let mut expect = vec![];

  let actual: Bytes = iter::repeat(b'x')
    .scan(100, |cnt, item| {
      if *cnt >= 1 {
        *cnt -= 1;
        expect.push(item);
        Some(item)
      } else {
        None
      }
    })
    .collect();

  assert_eq!(&actual[..], &expect[..]);
}

#[test]
fn bytes_buf_mut_advance() {
  let mut bytes = BytesMut::with_capacity(1024);

  unsafe {
    let ptr = bytes.chunk_mut().as_mut_ptr();
    assert_eq!(1024, bytes.chunk_mut().len());

    bytes.advance_mut(10);

    let next = bytes.chunk_mut().as_mut_ptr();
    assert_eq!(1024 - 10, bytes.chunk_mut().len());
    assert_eq!(ptr.offset(10), next);

    // advance to the end
    bytes.advance_mut(1024 - 10);

    // The buffer size is doubled
    assert_eq!(1024, bytes.chunk_mut().len());
  }
}

#[test]
fn bytes_buf_mut_reuse_when_fully_consumed() {
  use bytes::{Buf, BytesMut};
  let mut buf = BytesMut::new();
  buf.reserve(8192);
  buf.extend_from_slice(&[0u8; 100][..]);

  let p = &buf[0] as *const u8;
  buf.advance(100);

  buf.reserve(8192);
  buf.extend_from_slice(b" ");

  assert_eq!(&buf[0] as *const u8, p);
}

#[test]
#[should_panic]
fn bytes_reserve_overflow() {
  let mut bytes = BytesMut::with_capacity(1024);
  bytes.put_slice(b"hello world");

  bytes.reserve(usize::MAX);
}

#[test]
fn bytes_with_capacity_but_empty() {
  // See https://github.com/tokio-rs/bytes/issues/340
  let vec = Vec::with_capacity(1);
  let _ = Bytes::from(vec);
}

#[test]
fn bytes_put_bytes() {
  let mut bytes = BytesMut::new();
  bytes.put_u8(17);
  bytes.put_bytes(19, 2);
  assert_eq!([17, 19, 19], bytes.as_ref());
}

#[test]
fn box_slice_empty() {
  // See https://github.com/tokio-rs/bytes/issues/340
  let empty: Box<[u8]> = Default::default();
  let b = Bytes::from(empty);
  assert!(b.is_empty());
}

#[test]
fn bytes_into_vec() {
  // Test kind == KIND_VEC
  let content = b"helloworld";

  let mut bytes = BytesMut::new();
  bytes.put_slice(content);

  let vec: Vec<u8> = bytes.into();
  assert_eq!(&vec, content);

  // Test kind == KIND_ARC, shared.is_unique() == True
  let mut bytes = BytesMut::new();
  bytes.put_slice(b"abcdewe23");
  bytes.put_slice(content);
  bytes.make_heap();

  // Overwrite the bytes to make sure only one reference to the underlying
  // Vec exists.
  bytes = bytes.split_off(9).unwrap();

  let vec: Vec<u8> = bytes.into();
  assert_eq!(&vec, content);

  // Test kind == KIND_ARC, shared.is_unique() == False
  let prefix = b"abcdewe23";

  let mut bytes = BytesMut::new();
  bytes.put_slice(prefix);
  bytes.put_slice(content);

  bytes.make_heap();

  let vec: Vec<u8> = bytes.split_off(prefix.len()).unwrap().into();
  assert_eq!(&vec, content);

  let vec: Vec<u8> = bytes.into();
  assert_eq!(&vec, prefix);
}

#[test]
fn test_bytes_into_vec() {
  // Test STATIC_VTABLE.to_vec
  let bs = b"1b23exfcz3r";
  let vec: Vec<u8> = Bytes::from_static(bs).into();
  assert_eq!(&*vec, bs);

  // Test bytes_mut.SHARED_VTABLE.to_vec impl
  eprintln!("1");
  let mut bytes_mut: BytesMut = bs[..].into();

  // Set kind to KIND_ARC so that after freeze, Bytes will use bytes_mut.SHARED_VTABLE
  eprintln!("2");
  drop(bytes_mut.split_off(bs.len()));

  eprintln!("3");
  let b1 = bytes_mut.freeze_shared();
  eprintln!("4");
  let b2 = b1.clone();

  eprintln!("{:#?}", (*b1).as_ptr());

  // shared.is_unique() = False
  eprintln!("5");
  assert_eq!(&*Vec::from(b2), bs);

  // shared.is_unique() = True
  eprintln!("6");
  assert_eq!(&*Vec::from(b1), bs);

  // Test bytes_mut.SHARED_VTABLE.to_vec impl where offset != 0
  let mut bytes_mut1: BytesMut = bs[..].into();
  bytes_mut1.make_heap();
  let bytes_mut2 = bytes_mut1.split_off(9).unwrap();

  let b1 = bytes_mut1.freeze_shared();
  let b2 = bytes_mut2.freeze_shared();

  assert_eq!(Vec::from(b2), bs[9..]);
  assert_eq!(Vec::from(b1), bs[..9]);
}

#[test]
fn test_bytes_into_vec_promotable_even() {
  let vec = vec![33u8; 1024];

  // Test cases where kind == KIND_VEC
  let b1 = Bytes::from(vec.clone());
  assert_eq!(Vec::from(b1), vec);

  // Test cases where kind == KIND_ARC, ref_cnt == 1
  let b1 = Bytes::from(vec.clone());
  drop(b1.clone());
  assert_eq!(Vec::from(b1), vec);

  // Test cases where kind == KIND_ARC, ref_cnt == 2
  let b1 = Bytes::from(vec.clone());
  let b2 = b1.clone();
  assert_eq!(Vec::from(b1), vec);

  // Test cases where vtable = SHARED_VTABLE, kind == KIND_ARC, ref_cnt == 1
  assert_eq!(Vec::from(b2), vec);

  // Test cases where offset != 0
  let mut b1 = Bytes::from(vec.clone());
  let b2 = b1.split_off(20);

  assert_eq!(Vec::from(b2), vec[20..]);
  assert_eq!(Vec::from(b1), vec[..20]);
}

#[test]
fn test_bytes_vec_conversion() {
  let mut vec = Vec::with_capacity(10);
  vec.extend(b"abcdefg");
  let b = Bytes::from(vec);
  let v = Vec::from(b);
  assert_eq!(v.len(), 7);
  assert_eq!(v.capacity(), 7);

  let mut b = Bytes::from(v);
  b.advance(1);
  let v = Vec::from(b);
  assert_eq!(v.len(), 6);
  assert_eq!(v.capacity(), 6);
  assert_eq!(v.as_slice(), b"bcdefg");
}

#[test]
fn test_bytes_mut_conversion() {
  let mut b1 = BytesMut::with_capacity(10);
  b1.extend(b"abcdefg");
  let b2 = Bytes::from(b1);
  let v = Vec::from(b2);
  assert_eq!(v.len(), 7);
  assert_eq!(v.capacity(), 7);

  let mut b = Bytes::from(v);
  b.advance(1);
  let v = Vec::from(b);
  assert_eq!(v.len(), 6);
  assert_eq!(v.capacity(), 6);
  assert_eq!(v.as_slice(), b"bcdefg");
}

#[test]
fn test_bytes_capacity_len() {
  for cap in 0..100 {
    for len in 0..=cap {
      let mut v = Vec::with_capacity(cap);
      v.resize(len, 0);
      let _ = Bytes::from(v);
    }
  }
}

#[test]
fn static_is_unique() {
  let b = Bytes::from_static(LONG);
  assert!(!b.is_unique());
}

#[test]
fn vec_is_unique() {
  let v: Vec<u8> = LONG.to_vec();
  let b = Bytes::from(v);
  assert!(b.is_unique());
}

#[test]
fn arc_is_unique() {
  let v: Vec<u8> = LONG.to_vec();
  let b = Bytes::from(v);
  let c = b.clone();
  assert!(!b.is_unique());
  drop(c);
  assert!(b.is_unique());
}

#[test]
fn shared_is_unique() {
  let v: Vec<u8> = LONG.to_vec();
  let b = Bytes::from(v);
  let c = b.clone();
  assert!(!c.is_unique());
  drop(b);
  assert!(c.is_unique());
}

#[test]
fn mut_shared_is_unique() {
  let mut b = BytesMut::from(LONG);
  b.make_heap();
  let c = b.split().unwrap().freeze_shared();
  assert!(!c.is_unique());
  drop(b);
  assert!(c.is_unique());
}

#[test]
fn test_bytesmut_from_bytes_static() {
  let bs = b"1b23exfcz3r";

  // Test STATIC_VTABLE.to_mut
  let bytes_mut = BytesMut::from(Bytes::from_static(bs));
  assert_eq!(bytes_mut, bs[..]);
}

#[test]
fn test_bytesmut_from_bytes_bytes_mut_vec() {
  let bs = b"1b23exfcz3r";
  let bs_long = b"1b23exfcz3r1b23exfcz3r";

  // Test case where kind == KIND_VEC
  let mut bytes_mut: BytesMut = bs[..].into();
  bytes_mut = BytesMut::from(bytes_mut.freeze_shared());
  assert_eq!(bytes_mut, bs[..]);
  bytes_mut.extend_from_slice(&bs[..]);
  assert_eq!(bytes_mut, bs_long[..]);
}

#[test]
fn test_bytesmut_from_bytes_bytes_mut_shared() {
  let bs = b"1b23exfcz3r";

  // Set kind to KIND_ARC so that after freeze_shared, Bytes will use bytes_mut.SHARED_VTABLE
  let mut bytes_mut: BytesMut = bs[..].into();
  drop(bytes_mut.split_off(bs.len()));

  let b1 = bytes_mut.freeze_shared();
  let b2 = b1.clone();

  // shared.is_unique() = False
  let mut b1m = BytesMut::from(b1);
  assert_eq!(b1m, bs[..]);
  b1m[0] = b'9';

  // shared.is_unique() = True
  let b2m = BytesMut::from(b2);
  assert_eq!(b2m, bs[..]);
}

#[test]
fn test_bytesmut_from_bytes_bytes_mut_offset() {
  let bs = b"1b23exfcz3r";

  // Test bytes_mut.SHARED_VTABLE.to_mut impl where offset != 0
  let mut bytes_mut1: BytesMut = bs[..].into();
  bytes_mut1.make_heap();
  let bytes_mut2 = bytes_mut1.split_off(9).unwrap();

  let b1 = bytes_mut1.freeze_shared();
  let b2 = bytes_mut2.freeze_shared();

  let b1m = BytesMut::from(b1);
  let b2m = BytesMut::from(b2);

  assert_eq!(b2m, bs[9..]);
  assert_eq!(b1m, bs[..9]);
}

#[test]
fn test_bytesmut_from_bytes_promotable_even_vec() {
  let vec = vec![33u8; 1024];

  // Test case where kind == KIND_VEC
  let b1 = Bytes::from(vec.clone());
  let b1m = BytesMut::from(b1);
  assert_eq!(b1m, vec);
}

#[test]
fn test_bytesmut_from_bytes_promotable_even_arc_1() {
  let vec = vec![33u8; 1024];

  // Test case where kind == KIND_ARC, ref_cnt == 1
  let b1 = Bytes::from(vec.clone());
  drop(b1.clone());
  let b1m = BytesMut::from(b1);
  assert_eq!(b1m, vec);
}

#[test]
fn test_bytesmut_from_bytes_promotable_even_arc_2() {
  let vec = vec![33u8; 1024];

  // Test case where kind == KIND_ARC, ref_cnt == 2
  let b1 = Bytes::from(vec.clone());
  let b2 = b1.clone();
  let b1m = BytesMut::from(b1);
  assert_eq!(b1m, vec);

  // Test case where vtable = SHARED_VTABLE, kind == KIND_ARC, ref_cnt == 1
  let b2m = BytesMut::from(b2);
  assert_eq!(b2m, vec);
}

#[test]
fn test_bytesmut_from_bytes_promotable_even_arc_offset() {
  let vec = vec![33u8; 1024];

  // Test case where offset != 0
  let mut b1 = Bytes::from(vec.clone());
  let b2 = b1.split_off(20);
  let b1m = BytesMut::from(b1);
  let b2m = BytesMut::from(b2);

  assert_eq!(b2m, vec[20..]);
  assert_eq!(b1m, vec[..20]);
}

#[test]
fn try_reclaim_empty() {
  let mut buf = BytesMut::new();
  assert!(!buf.try_reclaim(64));
  buf.reserve(64);
  assert!(buf.try_reclaim(64));
  let cap = buf.capacity();
  assert!(cap >= 64);
  assert!(!buf.try_reclaim(cap + 1));

  let mut buf = BytesMut::new();
  buf.reserve(64);
  let cap = buf.capacity();
  assert!(cap >= 64);
  buf.make_heap();
  let mut split = buf.split().unwrap();
  drop(buf);
  assert_eq!(0, split.capacity());
  assert!(split.try_reclaim(64));
  assert!(!split.try_reclaim(cap + 1));
}

#[test]
fn try_reclaim_vec() {
  let mut buf = BytesMut::with_capacity(64);
  buf.put_slice(b"abc");
  // Reclaiming a ludicrous amount of space should calmly return false
  assert!(!buf.try_reclaim(usize::MAX));

  assert!(!buf.try_reclaim(64));
  buf.advance(2);
  assert_eq!(62, buf.capacity());
  // We can reclaim 64 bytes, because the byte in the buffer can be moved to the front. 63 bytes
  // cannot be reclaimed because there is already one byte stored
  assert!(!buf.try_reclaim(64));
  assert!(buf.try_reclaim(63));
  buf.advance(1);
  assert!(buf.try_reclaim(64));
  assert_eq!(64, buf.capacity());
}

#[test]
fn try_reclaim_arc() {
  let mut buf = BytesMut::with_capacity(64);
  buf.put_slice(b"abc");
  buf.make_heap();
  let x = buf.split().unwrap().freeze_shared();
  buf.put_slice(b"def");
  // Reclaiming a ludicrous amount of space should calmly return false
  assert!(!buf.try_reclaim(usize::MAX));

  let y = buf.split().unwrap().freeze_shared();
  let z = y.clone();
  assert!(!buf.try_reclaim(64));
  drop(x);
  drop(z);
  assert!(!buf.try_reclaim(64));
  drop(y);
  assert!(buf.try_reclaim(64));
  assert_eq!(64, buf.capacity());
  assert_eq!(0, buf.len());
  buf.put_slice(b"abc");
  buf.put_slice(b"def");
  assert_eq!(64, buf.capacity());
  assert_eq!(6, buf.len());
  assert!(!buf.try_reclaim(64));
  buf.advance(4);
  assert!(buf.try_reclaim(4));
  buf.advance(2);
  assert!(buf.try_reclaim(6));
}

#[test]
fn split_off_empty_addr_mut() {
  let mut buf = BytesMut::from([0; 1024].as_slice());

  let ptr_start = buf.as_ptr();
  let ptr_end = ptr_start.wrapping_add(1024);

  let empty_end = buf.split_off(1024).unwrap();
  assert_eq!(empty_end.len(), 0);
  assert_eq!(empty_end.as_ptr(), ptr_end);

  let _ = buf.split_off(0);
  assert_eq!(buf.len(), 0);
  assert_eq!(buf.as_ptr(), ptr_start);

  // Is miri happy about the provenance?
  let _ = &empty_end[..];
  let _ = &buf[..];
}

#[test]
fn split_to_empty_addr_mut() {
  let mut buf = BytesMut::from([0; 1024].as_slice());

  let ptr_start = buf.as_ptr();
  let ptr_end = ptr_start.wrapping_add(1024);

  let empty_start = buf.split_to(0).unwrap();
  assert_eq!(empty_start.len(), 0);
  assert_eq!(empty_start.as_ptr(), ptr_start);

  let _ = buf.split_to(1024);
  assert_eq!(buf.len(), 0);
  assert_eq!(buf.as_ptr(), ptr_end);

  // Is miri happy about the provenance?
  let _ = &empty_start[..];
  let _ = &buf[..];
}

#[derive(Clone)]
struct SharedAtomicCounter(Arc<AtomicUsize>);

impl SharedAtomicCounter {
  pub fn new() -> Self {
    SharedAtomicCounter(Arc::new(AtomicUsize::new(0)))
  }

  pub fn increment(&self) {
    self.0.fetch_add(1, Ordering::AcqRel);
  }

  pub fn get(&self) -> usize {
    self.0.load(Ordering::Acquire)
  }
}

#[derive(Clone)]
struct OwnedTester<const L: usize> {
  buf: [u8; L],
  drop_count: SharedAtomicCounter,
  pub panic_as_ref: bool,
}

impl<const L: usize> OwnedTester<L> {
  fn new(buf: [u8; L], drop_count: SharedAtomicCounter) -> Self {
    Self {
      buf,
      drop_count,
      panic_as_ref: false,
    }
  }
}

impl<const L: usize> AsRef<[u8]> for OwnedTester<L> {
  fn as_ref(&self) -> &[u8] {
    if self.panic_as_ref {
      panic!("test-triggered panic in `AsRef<[u8]> for OwnedTester`");
    }
    self.buf.as_slice()
  }
}

impl<const L: usize> Drop for OwnedTester<L> {
  fn drop(&mut self) {
    self.drop_count.increment();
  }
}

#[test]
fn owned_is_unique_always_false() {
  let b1 = Bytes::from_owner([1, 2, 3, 4, 5, 6, 7]);
  assert!(!b1.is_unique()); // even if ref_cnt == 1
  let b2 = b1.clone();
  assert!(!b1.is_unique());
  assert!(!b2.is_unique());
  drop(b1);
  assert!(!b2.is_unique()); // even if ref_cnt == 1
}

#[test]
fn owned_buf_sharing() {
  let buf = [1, 2, 3, 4, 5, 6, 7];
  let b1 = Bytes::from_owner(buf);
  let b2 = b1.clone();
  assert_eq!(&buf[..], &b1[..]);
  assert_eq!(&buf[..], &b2[..]);
  assert_eq!(b1.as_ptr(), b2.as_ptr());
  assert_eq!(b1.len(), b2.len());
  assert_eq!(b1.len(), buf.len());
}

#[test]
fn owned_buf_slicing() {
  let b1 = Bytes::from_owner(SHORT);
  assert_eq!(SHORT, &b1[..]);
  let b2 = b1.slice(1..(b1.len() - 1));
  assert_eq!(&SHORT[1..(SHORT.len() - 1)], b2);
  assert_eq!(unsafe { SHORT.as_ptr().add(1) }, b2.as_ptr());
  assert_eq!(SHORT.len() - 2, b2.len());
}

#[test]
fn owned_dropped_exactly_once() {
  let buf: [u8; 5] = [1, 2, 3, 4, 5];
  let drop_counter = SharedAtomicCounter::new();
  let owner = OwnedTester::new(buf, drop_counter.clone());
  let b1 = Bytes::from_owner(owner);
  let b2 = b1.clone();
  assert_eq!(drop_counter.get(), 0);
  drop(b1);
  assert_eq!(drop_counter.get(), 0);
  let b3 = b2.slice(1..b2.len() - 1);
  drop(b2);
  assert_eq!(drop_counter.get(), 0);
  drop(b3);
  assert_eq!(drop_counter.get(), 1);
}

#[test]
fn owned_to_mut() {
  let buf: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
  let drop_counter = SharedAtomicCounter::new();
  let owner = OwnedTester::new(buf, drop_counter.clone());
  let b1 = Bytes::from_owner(owner);

  // Holding an owner will fail converting to a BytesMut,
  // even when the bytes instance has a ref_cnt == 1.
  let b1 = b1.try_into_mut().unwrap_err();

  // That said, it's still possible, just not cheap.
  let bm1: BytesMut = b1.into();
  let new_buf = &bm1[..];
  assert_eq!(new_buf, &buf[..]);

  // `.into::<BytesMut>()` has correctly dropped the owner
  assert_eq!(drop_counter.get(), 1);
}

#[test]
fn owned_to_vec() {
  let buf: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
  let drop_counter = SharedAtomicCounter::new();
  let owner = OwnedTester::new(buf, drop_counter.clone());
  let b1 = Bytes::from_owner(owner);

  let v1 = b1.to_vec();
  assert_eq!(&v1[..], &buf[..]);
  assert_eq!(&v1[..], &b1[..]);

  drop(b1);
  assert_eq!(drop_counter.get(), 1);
}

#[test]
fn owned_into_vec() {
  let drop_counter = SharedAtomicCounter::new();
  let buf: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
  let owner = OwnedTester::new(buf, drop_counter.clone());
  let b1 = Bytes::from_owner(owner);

  let v1: Vec<u8> = b1.into();
  assert_eq!(&v1[..], &buf[..]);
  // into() vec will copy out of the owner and drop it
  assert_eq!(drop_counter.get(), 1);
}

#[test]
#[cfg_attr(not(panic = "unwind"), ignore)]
fn owned_safe_drop_on_as_ref_panic() {
  let buf: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
  let drop_counter = SharedAtomicCounter::new();
  let mut owner = OwnedTester::new(buf, drop_counter.clone());
  owner.panic_as_ref = true;

  let result = panic::catch_unwind(AssertUnwindSafe(|| {
    let _ = Bytes::from_owner(owner);
  }));

  assert!(result.is_err());
  assert_eq!(drop_counter.get(), 1);
}

// fn test_slice_ref(bytes: &Bytes, start: usize, end: usize, expected: &[u8]) {
//   let slice = &(bytes.as_ref()[start..end]);
//   let sub = bytes.slice_ref(slice);
//   assert_eq!(&sub[..], expected);
// }

// #[test]
// fn slice_ref_works() {
//   let bytes = Bytes::from(&b"012345678"[..]);

//   test_slice_ref(&bytes, 0, 0, b"");
//   test_slice_ref(&bytes, 0, 3, b"012");
//   test_slice_ref(&bytes, 2, 6, b"2345");
//   test_slice_ref(&bytes, 7, 9, b"78");
//   test_slice_ref(&bytes, 9, 9, b"");
// }

// #[test]
// fn slice_ref_empty() {
//   let bytes = Bytes::from(&b""[..]);
//   let slice = &(bytes.as_ref()[0..0]);

//   let sub = bytes.slice_ref(slice);
//   assert_eq!(&sub[..], b"");
// }

// #[test]
// fn slice_ref_empty_subslice() {
//   let bytes = Bytes::from(&b"abcde"[..]);
//   let subbytes = bytes.slice(0..0);
//   let slice = &subbytes[..];
//   // The `slice` object is derived from the original `bytes` object
//   // so `slice_ref` should work.
//   assert_eq!(Bytes::new(), bytes.slice_ref(slice));
// }

// #[test]
// #[should_panic]
// fn slice_ref_catches_not_a_subset() {
//   let bytes = Bytes::from(&b"012345678"[..]);
//   let slice = &b"012345"[0..4];

//   bytes.slice_ref(slice);
// }

// #[test]
// fn slice_ref_not_an_empty_subset() {
//   let bytes = Bytes::from(&b"012345678"[..]);
//   let slice = &b""[0..0];

//   assert_eq!(Bytes::new(), bytes.slice_ref(slice));
// }

// #[test]
// fn empty_slice_ref_not_an_empty_subset() {
//   let bytes = Bytes::new();
//   let slice = &b"some other slice"[0..0];

//   assert_eq!(Bytes::new(), bytes.slice_ref(slice));
// }
