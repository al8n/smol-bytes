#![warn(rust_2018_idioms)]

//! Coverage of the comparison / hash impls across every wrapper: `PartialEq` /
//! `PartialOrd` against `Self`, `[u8]`, `[u8; N]`, `&[u8]`, `str` / `&str`,
//! `String`, `Vec<u8>`, the smart-pointer slice/str types, `bytes::Bytes` /
//! `bytes::BytesMut`, and the cross-wrapper pairs. Each assertion checks a real
//! semantic property: symmetry of equality, ordering agreement with the
//! underlying slice/str, and hash consistency across inline/heap storage.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

use smol_bytes::{Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut, compact, shared};

fn hash_of<T: Hash + ?Sized>(v: &T) -> u64 {
  let mut h = DefaultHasher::new();
  v.hash(&mut h);
  h.finish()
}

/// Asserts `w == eq` and `w < gt` both ways, plus explicit `partial_cmp`.
macro_rules! cmp_pair {
  ($w:expr, $eq:expr, $gt:expr) => {{
    let w = &$w;
    assert_eq!(*w, $eq, "wrapper == equal (forward)");
    assert_eq!($eq, *w, "equal == wrapper (reverse)");
    assert_ne!(*w, $gt, "wrapper != greater (forward)");
    assert_ne!($gt, *w, "greater != wrapper (reverse)");
    assert!(*w < $gt, "wrapper < greater (forward)");
    assert!($gt > *w, "greater > wrapper (reverse)");
    assert_eq!(w.partial_cmp(&$eq), Some(::core::cmp::Ordering::Equal));
    assert_eq!(w.partial_cmp(&$gt), Some(::core::cmp::Ordering::Less));
  }};
}

/// Asserts `w == eq` / `w != ne` both ways (for types without an ordering impl).
macro_rules! eq_pair {
  ($w:expr, $eq:expr, $ne:expr) => {{
    let w = &$w;
    assert_eq!(*w, $eq);
    assert_eq!($eq, *w);
    assert_ne!(*w, $ne);
    assert_ne!($ne, *w);
  }};
}

// ===========================================================================
// Buffer
// ===========================================================================

#[test]
fn buffer_self_eq_ord_hash() {
  let a = Buffer::from(*b"hello");
  let a2 = Buffer::from(*b"hello");
  let b = Buffer::from(*b"hellp");

  assert_eq!(a, a2);
  assert_ne!(a, b);
  assert!(a < b);
  assert!(b > a);
  assert_eq!(a.cmp(&a2), core::cmp::Ordering::Equal);
  assert_eq!(a.partial_cmp(&b), Some(core::cmp::Ordering::Less));
  assert_eq!(hash_of(&a), hash_of(&a2));
  assert_eq!(hash_of(&a), hash_of(b"hello".as_slice()));
}

#[test]
fn buffer_vs_slices_and_arrays() {
  let buf = Buffer::from(*b"hello");
  let eq_sl: &[u8] = b"hello";
  let gt_sl: &[u8] = b"hellp";

  // [u8] (unsized slice), both directions, eq + ordering.
  cmp_pair!(buf, *eq_sl, *gt_sl);
  // &[u8] reference, exercises the blanket &T impl and its reverse.
  cmp_pair!(buf, eq_sl, gt_sl);
  // [u8; N] arrays.
  cmp_pair!(buf, *b"hello", *b"hellp");
}

#[test]
fn buffer_vs_str_types() {
  let buf = Buffer::from(*b"hello");
  // str (unsized) and &str.
  cmp_pair!(buf, *"hello", *"hellp");
  cmp_pair!(buf, "hello", "hellp");
  // String / Box<str> / Rc<str> / Arc<str>.
  cmp_pair!(buf, String::from("hello"), String::from("hellp"));
  cmp_pair!(buf, Box::<str>::from("hello"), Box::<str>::from("hellp"));
  cmp_pair!(buf, Rc::<str>::from("hello"), Rc::<str>::from("hellp"));
  cmp_pair!(buf, Arc::<str>::from("hello"), Arc::<str>::from("hellp"));
}

#[test]
fn buffer_vs_owned_byte_containers() {
  let buf = Buffer::from(*b"hello");
  cmp_pair!(buf, b"hello".to_vec(), b"hellp".to_vec());
  cmp_pair!(
    buf,
    Box::<[u8]>::from(&b"hello"[..]),
    Box::<[u8]>::from(&b"hellp"[..])
  );
  cmp_pair!(
    buf,
    Rc::<[u8]>::from(&b"hello"[..]),
    Rc::<[u8]>::from(&b"hellp"[..])
  );
  cmp_pair!(
    buf,
    Arc::<[u8]>::from(&b"hello"[..]),
    Arc::<[u8]>::from(&b"hellp"[..])
  );
}

#[test]
fn buffer_vs_bytes_crate_and_wrappers() {
  let buf = Buffer::from(*b"hello");
  // external bytes crate types
  cmp_pair!(
    buf,
    bytes::Bytes::from_static(b"hello"),
    bytes::Bytes::from_static(b"hellp")
  );
  cmp_pair!(
    buf,
    bytes::BytesMut::from(&b"hello"[..]),
    bytes::BytesMut::from(&b"hellp"[..])
  );
  // smol-bytes RawBytes wrappers (shared + compact strategy)
  cmp_pair!(
    buf,
    shared::Bytes::from_static(b"hello"),
    shared::Bytes::from_static(b"hellp")
  );
  cmp_pair!(
    buf,
    compact::Bytes::from_static(b"hello"),
    compact::Bytes::from_static(b"hellp")
  );
}

// ===========================================================================
// BytesMut
// ===========================================================================

#[test]
fn bytes_mut_self_eq_ord_hash_inline_and_heap() {
  let a = BytesMut::from(&b"hello"[..]);
  let a2 = BytesMut::from(&b"hello"[..]);
  let b = BytesMut::from(&b"hellp"[..]);

  assert_eq!(a, a2);
  assert_ne!(a, b);
  assert!(a < b);
  assert!(b > a);
  assert_eq!(hash_of(&a), hash_of(&a2));

  // Equal content across inline vs heap storage must hash equal.
  let inline = BytesMut::from(&b"hello"[..]);
  assert!(inline.is_inline());
  let mut heap = BytesMut::with_capacity(128);
  heap.extend_from_slice(b"hello");
  assert!(heap.is_heap());
  assert_eq!(inline, heap);
  assert_eq!(hash_of(&inline), hash_of(&heap));
}

#[test]
fn bytes_mut_vs_slices_arrays_and_strs() {
  let buf = BytesMut::from(&b"hello"[..]);
  let eq_sl: &[u8] = b"hello";
  let gt_sl: &[u8] = b"hellp";

  cmp_pair!(buf, *eq_sl, *gt_sl);
  cmp_pair!(buf, eq_sl, gt_sl);
  cmp_pair!(buf, *b"hello", *b"hellp");
  cmp_pair!(buf, *"hello", *"hellp");
  cmp_pair!(buf, "hello", "hellp");
}

#[test]
fn bytes_mut_vs_owned_and_wrappers() {
  let buf = BytesMut::from(&b"hello"[..]);
  cmp_pair!(buf, String::from("hello"), String::from("hellp"));
  cmp_pair!(buf, b"hello".to_vec(), b"hellp".to_vec());
  cmp_pair!(
    buf,
    Box::<[u8]>::from(&b"hello"[..]),
    Box::<[u8]>::from(&b"hellp"[..])
  );
  cmp_pair!(buf, Rc::<str>::from("hello"), Rc::<str>::from("hellp"));
  cmp_pair!(buf, Arc::<str>::from("hello"), Arc::<str>::from("hellp"));
  cmp_pair!(
    buf,
    bytes::Bytes::from_static(b"hello"),
    bytes::Bytes::from_static(b"hellp")
  );
  cmp_pair!(
    buf,
    bytes::BytesMut::from(&b"hello"[..]),
    bytes::BytesMut::from(&b"hellp"[..])
  );
  // Cross with the immutable RawBytes wrappers.
  cmp_pair!(
    buf,
    shared::Bytes::from_static(b"hello"),
    shared::Bytes::from_static(b"hellp")
  );
  cmp_pair!(
    buf,
    compact::Bytes::from_static(b"hello"),
    compact::Bytes::from_static(b"hellp")
  );
}

// ===========================================================================
// shared::Bytes / compact::Bytes (RawBytes)
// ===========================================================================

// Builds a heap-backed shared::Bytes whose visible content is `prefix` (<= 62
// bytes) by slicing a larger heap allocation — the shared strategy keeps the
// heap backing for non-empty slices.
fn shared_heap_small(prefix: &[u8]) -> shared::Bytes {
  let mut data = prefix.to_vec();
  data.extend(std::iter::repeat_n(b'X', 64));
  let big = shared::Bytes::copy_from_slice(&data);
  assert!(big.is_heap());
  let small = big.slice(0..prefix.len());
  assert!(small.is_heap());
  small
}

#[test]
fn shared_bytes_self_eq_ptr_and_slice_paths_hash() {
  // ptr_eq fast path: a clone shares the heap allocation.
  let heap = shared::Bytes::from(vec![7u8; 100]);
  let clone = heap.clone();
  assert_eq!(heap, clone);

  // slice-compare path: two independent heap buffers with identical content.
  let other = shared::Bytes::from(vec![7u8; 100]);
  assert_eq!(heap, other);

  let a = shared::Bytes::from_static(b"hello");
  let b = shared::Bytes::from_static(b"hellp");
  assert!(a < b);
  assert!(b > a);

  // inline vs heap of equal content: equal and hash-equal.
  let inline = shared::Bytes::from_static(b"hello");
  assert!(inline.is_inline());
  let heap_small = shared_heap_small(b"hello");
  assert_eq!(inline, heap_small);
  assert_eq!(hash_of(&inline), hash_of(&heap_small));
  assert_eq!(hash_of(&inline), hash_of(b"hello".as_slice()));
}

#[test]
fn shared_bytes_vs_all_targets() {
  let buf = shared::Bytes::from_static(b"hello");
  let eq_sl: &[u8] = b"hello";
  let gt_sl: &[u8] = b"hellp";

  cmp_pair!(buf, *eq_sl, *gt_sl);
  cmp_pair!(buf, eq_sl, gt_sl);
  cmp_pair!(buf, *b"hello", *b"hellp");
  cmp_pair!(buf, *"hello", *"hellp");
  cmp_pair!(buf, "hello", "hellp");
  cmp_pair!(buf, String::from("hello"), String::from("hellp"));
  cmp_pair!(buf, b"hello".to_vec(), b"hellp".to_vec());
  cmp_pair!(
    buf,
    Box::<[u8]>::from(&b"hello"[..]),
    Box::<[u8]>::from(&b"hellp"[..])
  );
  cmp_pair!(
    buf,
    Rc::<[u8]>::from(&b"hello"[..]),
    Rc::<[u8]>::from(&b"hellp"[..])
  );
  cmp_pair!(
    buf,
    Arc::<[u8]>::from(&b"hello"[..]),
    Arc::<[u8]>::from(&b"hellp"[..])
  );
  cmp_pair!(buf, Box::<str>::from("hello"), Box::<str>::from("hellp"));
  cmp_pair!(buf, Rc::<str>::from("hello"), Rc::<str>::from("hellp"));
  cmp_pair!(buf, Arc::<str>::from("hello"), Arc::<str>::from("hellp"));
  cmp_pair!(
    buf,
    bytes::Bytes::from_static(b"hello"),
    bytes::Bytes::from_static(b"hellp")
  );
  cmp_pair!(
    buf,
    bytes::BytesMut::from(&b"hello"[..]),
    bytes::BytesMut::from(&b"hellp"[..])
  );
}

#[test]
fn compact_bytes_self_and_targets() {
  let a = compact::Bytes::from_static(b"hello");
  let a2 = compact::Bytes::from_static(b"hello");
  let b = compact::Bytes::from_static(b"hellp");
  assert_eq!(a, a2);
  assert!(a < b);
  assert!(b > a);
  assert_eq!(hash_of(&a), hash_of(&a2));

  let eq_sl: &[u8] = b"hello";
  let gt_sl: &[u8] = b"hellp";
  cmp_pair!(a, *eq_sl, *gt_sl);
  cmp_pair!(a, eq_sl, gt_sl);
  cmp_pair!(a, *b"hello", *b"hellp");
  cmp_pair!(a, "hello", "hellp");
  cmp_pair!(a, String::from("hello"), String::from("hellp"));
  cmp_pair!(a, b"hello".to_vec(), b"hellp".to_vec());
  cmp_pair!(
    a,
    bytes::Bytes::from_static(b"hello"),
    bytes::Bytes::from_static(b"hellp")
  );
}

// ===========================================================================
// UTF-8 wrappers
// ===========================================================================

#[test]
fn utf8_buffer_cmp_and_cross() {
  let buf = Utf8Buffer::from("hello");
  let same = Utf8Buffer::from("hello");
  let bigger = Utf8Buffer::from("hellp");

  assert_eq!(buf, same);
  assert!(buf < bigger);
  assert!(bigger > buf);
  assert_eq!(buf.cmp(&same), core::cmp::Ordering::Equal);
  assert_eq!(hash_of(&buf), hash_of(&same));
  assert_eq!(hash_of(&buf), hash_of("hello"));

  cmp_pair!(buf, *"hello", *"hellp");
  // Utf8 wrappers implement PartialEq<&str> but not PartialOrd<&str>.
  assert_eq!(buf, "hello");
  assert_eq!("hello", buf);
  assert_ne!(buf, "world");
  cmp_pair!(buf, String::from("hello"), String::from("hellp"));

  // cross-wrapper equality (no ordering impls for these pairs)
  eq_pair!(buf, Utf8Bytes::from("hello"), Utf8Bytes::from("world"));
  eq_pair!(
    buf,
    compact::Utf8Bytes::from("hello"),
    compact::Utf8Bytes::from("world")
  );
  eq_pair!(
    buf,
    Utf8BytesMut::from("hello"),
    Utf8BytesMut::from("world")
  );
}

#[test]
fn utf8_bytes_cmp_and_cross_inline_heap() {
  let buf = Utf8Bytes::from("hello");
  let same = Utf8Bytes::from("hello");
  let bigger = Utf8Bytes::from("hellp");

  assert_eq!(buf, same);
  assert!(buf < bigger);
  assert!(bigger > buf);
  assert_eq!(hash_of(&buf), hash_of(&same));

  cmp_pair!(buf, *"hello", *"hellp");
  // Utf8 wrappers implement PartialEq<&str> but not PartialOrd<&str>.
  assert_eq!(buf, "hello");
  assert_eq!("hello", buf);
  assert_ne!(buf, "world");
  cmp_pair!(buf, String::from("hello"), String::from("hellp"));

  // cross-wrapper equality
  eq_pair!(
    buf,
    Utf8BytesMut::from("hello"),
    Utf8BytesMut::from("world")
  );
  eq_pair!(buf, Utf8Buffer::from("hello"), Utf8Buffer::from("world"));

  // inline vs heap of equal content hash equal (shared keeps heap on slice).
  let long = "hello".to_string() + &"z".repeat(80);
  let heap = Utf8Bytes::from(long.as_str());
  assert!(heap.len() > 62);
  let heap_small = heap.slice(0..5);
  assert_eq!(heap_small.as_str(), "hello");
  assert_eq!(buf, heap_small);
  assert_eq!(hash_of(&buf), hash_of(&heap_small));
}

#[test]
fn utf8_bytes_mut_cmp_and_cross() {
  let buf = Utf8BytesMut::from("hello");
  let same = Utf8BytesMut::from("hello");
  let bigger = Utf8BytesMut::from("hellp");

  assert_eq!(buf, same);
  assert!(buf < bigger);
  assert!(bigger > buf);
  assert_eq!(hash_of(&buf), hash_of(&same));

  cmp_pair!(buf, *"hello", *"hellp");
  // Utf8 wrappers implement PartialEq<&str> but not PartialOrd<&str>.
  assert_eq!(buf, "hello");
  assert_eq!("hello", buf);
  assert_ne!(buf, "world");
  cmp_pair!(buf, String::from("hello"), String::from("hellp"));

  eq_pair!(buf, Utf8Bytes::from("hello"), Utf8Bytes::from("world"));
  eq_pair!(buf, Utf8Buffer::from("hello"), Utf8Buffer::from("world"));
}
