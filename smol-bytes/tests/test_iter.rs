#![warn(rust_2018_idioms)]

use bytes::buf::IntoIter;

#[test]
fn iter_len() {
  use smol_bytes::Bytes;
  let buf = Bytes::from_static(b"hello world");
  let iter = IntoIter::new(buf);

  assert_eq!(iter.size_hint(), (11, Some(11)));
  assert_eq!(iter.len(), 11);
}

#[test]
fn empty_iter_len() {
  use smol_bytes::Bytes;
  let buf = Bytes::new();
  let iter = IntoIter::new(buf);

  assert_eq!(iter.size_hint(), (0, Some(0)));
  assert_eq!(iter.len(), 0);
}

#[test]
fn iter_len_compact() {
  use smol_bytes::compact::Bytes;

  let buf = Bytes::from_static(b"hello world");
  let iter = IntoIter::new(buf);

  assert_eq!(iter.size_hint(), (11, Some(11)));
  assert_eq!(iter.len(), 11);
}

#[test]
fn empty_iter_len_compact() {
  use smol_bytes::compact::Bytes;

  let buf = Bytes::new();
  let iter = IntoIter::new(buf);

  assert_eq!(iter.size_hint(), (0, Some(0)));
  assert_eq!(iter.len(), 0);
}
