#![warn(rust_2018_idioms)]

use core::{borrow::BorrowMut, ops::DerefMut};
use std::{
  borrow::Cow,
  panic::{AssertUnwindSafe, catch_unwind},
};
#[cfg(feature = "std")]
use std::{ffi::OsStr, path::Path};

use smol_bytes::{
  Bytes, BytesMut, Utf8BufMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut, compact, shared,
};

#[test]
fn mutable_utf8_wrappers_expose_safe_mutable_str_traits() {
  fn mutate<T>(value: &mut T)
  where
    T: AsMut<str> + BorrowMut<str> + DerefMut<Target = str> + Utf8BufMut,
  {
    value.as_mut().make_ascii_uppercase();
    value.borrow_mut().make_ascii_lowercase();
    <T as Utf8BufMut>::as_mut_str(value).make_ascii_uppercase();
    value.deref_mut().make_ascii_lowercase();
  }

  let mut fixed = Utf8Buffer::try_from("abc").unwrap();
  mutate(&mut fixed);
  fixed[0..3].make_ascii_uppercase();
  assert_eq!(fixed.as_str(), "ABC");

  let mut growable = Utf8BytesMut::from("def");
  mutate(&mut growable);
  growable[0..3].make_ascii_uppercase();
  assert_eq!(growable.as_str(), "DEF");

  let mut multibyte = Utf8BytesMut::from("café");
  multibyte[0..3].make_ascii_uppercase();
  assert_eq!(multibyte.as_str(), "CAFé");
  let boundary_error = catch_unwind(AssertUnwindSafe(|| {
    let _: &mut str = &mut multibyte[0..4];
  }));
  assert!(boundary_error.is_err());
  assert_eq!(multibyte.as_str(), "CAFé");

  let mut fixed_multibyte = Utf8Buffer::try_from("café").unwrap();
  let boundary_error = catch_unwind(AssertUnwindSafe(|| {
    let _: &mut str = &mut fixed_multibyte[0..4];
  }));
  assert!(boundary_error.is_err());
  assert_eq!(fixed_multibyte.as_str(), "café");
}

#[test]
fn utf8_bytes_mutates_only_its_view_and_recovers_mutable_storage() {
  let mut view = Utf8Bytes::from("hello café");
  let original = view.clone();
  view.advance(6);
  assert_eq!(view.as_str(), "café");
  view.try_truncate(3).unwrap();
  assert_eq!(view.as_str(), "caf");
  assert_eq!(original.as_str(), "hello café");

  let mut boundary = Utf8Bytes::from("éx");
  assert!(boundary.try_advance(1).is_err());
  assert_eq!(boundary.as_str(), "éx");
  boundary.try_advance(2).unwrap();
  assert_eq!(boundary.as_str(), "x");
  boundary.clear();
  assert!(boundary.is_empty());

  let unique = Utf8Bytes::from("x".repeat(80));
  assert!(unique.is_unique());
  let mut mutable = match unique.try_into_mut() {
    Ok(value) => value,
    Err(_) => panic!("unique UTF-8 bytes should become mutable"),
  };
  mutable.as_mut_str().make_ascii_uppercase();
  assert_eq!(mutable.as_str(), "X".repeat(80));

  let shared = Utf8Bytes::from("y".repeat(80));
  let clone = shared.clone();
  assert!(!shared.is_unique());
  let recovered = match shared.try_into_mut() {
    Ok(_) => panic!("shared UTF-8 bytes should preserve the immutable view"),
    Err(value) => value,
  };
  assert_eq!(recovered.as_str(), clone.as_str());
  assert!(recovered.is_heap());
}

#[test]
fn default_freeze_and_cross_strategy_conversions_preserve_large_heap_backing() {
  let raw: Bytes = BytesMut::from(vec![b'a'; 100]).freeze();
  assert!(raw.is_heap());

  let text: Utf8Bytes = Utf8BytesMut::from("x".repeat(100)).freeze();
  assert!(text.is_heap());

  let shared_bytes = shared::Bytes::from(vec![b'a'; 100]);
  let shared_pointer = shared_bytes.as_slice().as_ptr();
  let compact_bytes: compact::Bytes = shared_bytes.into();
  assert!(compact_bytes.is_heap());
  assert_eq!(compact_bytes.as_slice().as_ptr(), shared_pointer);

  let compact_bytes = compact::Bytes::from(vec![b'b'; 100]);
  let compact_pointer = compact_bytes.as_slice().as_ptr();
  let shared_bytes: shared::Bytes = compact_bytes.into();
  assert!(shared_bytes.is_heap());
  assert_eq!(shared_bytes.as_slice().as_ptr(), compact_pointer);

  let shared_compare = shared::Bytes::from_static(b"abc");
  let compact_compare = compact::Bytes::from_static(b"abd");
  assert_ne!(shared_compare, compact_compare);
  assert_eq!(
    shared_compare.partial_cmp(&compact_compare),
    Some(core::cmp::Ordering::Less)
  );

  let shared_text = shared::Utf8Bytes::from("c".repeat(100));
  let pointer = shared_text.as_str().as_ptr();
  let compact_text: compact::Utf8Bytes = shared_text.into();
  assert_eq!(compact_text.as_str().as_ptr(), pointer);
  assert_eq!(compact_text, shared::Utf8Bytes::from("c".repeat(100)));
  assert_eq!(
    compact_text.partial_cmp(&shared::Utf8Bytes::from("d".repeat(100))),
    Some(core::cmp::Ordering::Less)
  );

  let compact_text = compact::Utf8Bytes::from("d".repeat(100));
  let pointer = compact_text.as_str().as_ptr();
  let shared_text: shared::Utf8Bytes = compact_text.into();
  assert_eq!(shared_text.as_str().as_ptr(), pointer);

  let mutable = Utf8BytesMut::from("convert me");
  let compact: compact::Utf8Bytes = mutable.into();
  let mutable: Utf8BytesMut = compact.into();
  assert_eq!(mutable.as_str(), "convert me");

  let fixed = Utf8Buffer::try_from("abc").unwrap();
  let mutable = Utf8BytesMut::from("abd");
  assert_eq!(fixed.partial_cmp(&mutable), Some(core::cmp::Ordering::Less));
  assert_eq!(
    mutable.partial_cmp(&fixed),
    Some(core::cmp::Ordering::Greater)
  );

  let immutable = Utf8Bytes::from("abd");
  assert_eq!(
    fixed.partial_cmp(&immutable),
    Some(core::cmp::Ordering::Less)
  );
  assert_eq!(
    immutable.partial_cmp(&fixed),
    Some(core::cmp::Ordering::Greater)
  );
  assert_eq!(
    immutable.partial_cmp(&mutable),
    Some(core::cmp::Ordering::Equal)
  );
  assert_eq!(
    mutable.partial_cmp(&immutable),
    Some(core::cmp::Ordering::Equal)
  );
}

#[test]
fn selected_owned_conversions_recover_errors_and_collect_chunks() {
  let raw = BytesMut::from(Cow::Owned(vec![1, 2, 3]));
  assert_eq!(raw.as_slice(), [1, 2, 3]);

  let invalid = vec![b'a', 0xff];
  let error = match Utf8Bytes::try_from(invalid) {
    Ok(_) => panic!("invalid UTF-8 unexpectedly validated"),
    Err(error) => error,
  };
  assert_eq!(error.into_bytes(), vec![b'a', 0xff]);

  let invalid = bytes::Bytes::from_static(b"\xff");
  let error = match Utf8BytesMut::try_from(invalid) {
    Ok(_) => panic!("invalid UTF-8 unexpectedly validated"),
    Err(error) => error,
  };
  assert_eq!(error.into_bytes().as_ref(), b"\xff");

  let incoming = bytes::BytesMut::from("m".repeat(80).as_bytes());
  let pointer = incoming.as_ptr();
  let text = match Utf8BytesMut::try_from(incoming) {
    Ok(value) => value,
    Err(_) => panic!("valid external bytes::BytesMut unexpectedly rejected"),
  };
  assert_eq!(text.as_str().as_ptr(), pointer);

  let incoming = bytes::BytesMut::from(&b"bad\xff"[..]);
  let pointer = incoming.as_ptr();
  let capacity = incoming.capacity();
  let error: smol_bytes::FromUtf8Error<bytes::BytesMut> = match Utf8BytesMut::try_from(incoming) {
    Ok(_) => panic!("invalid external bytes::BytesMut unexpectedly validated"),
    Err(error) => error,
  };
  assert_eq!(error.as_bytes().as_ptr(), pointer);
  let recovered = error.into_bytes();
  assert_eq!(recovered.as_ptr(), pointer);
  assert_eq!(recovered.capacity(), capacity);
  assert_eq!(recovered.as_ref(), b"bad\xff");

  let incoming = bytes::Bytes::from("q".repeat(80));
  let pointer = incoming.as_ptr();
  let text = shared::Utf8Bytes::try_from(incoming).unwrap();
  assert_eq!(text.as_str().as_ptr(), pointer);

  let chunks = vec![
    bytes::Bytes::from_static(b"ab"),
    bytes::Bytes::from_static(b"cd"),
  ];
  let collected: BytesMut = chunks.into_iter().collect();
  assert_eq!(collected.as_slice(), b"abcd");

  let chunks = vec![
    bytes::Bytes::from_static(b"ab"),
    bytes::Bytes::from_static(b"cd"),
  ];
  let collected: shared::Bytes = chunks.into_iter().collect();
  assert_eq!(collected.as_slice(), b"abcd");

  let chunks = vec![
    shared::Bytes::from_static(b"ab"),
    shared::Bytes::from_static(b"cd"),
  ];
  let collected: compact::Bytes = chunks.into_iter().collect();
  assert_eq!(collected.as_slice(), b"abcd");

  let text = Utf8Bytes::from(Cow::Owned("hello".to_owned()));
  let bytes: bytes::BytesMut = text.into();
  assert_eq!(bytes.as_ref(), b"hello");

  let bytes = shared::Bytes::from(vec![b'z'; 80]);
  let boxed: Box<[u8]> = bytes.into();
  assert_eq!(boxed.as_ref(), &[b'z'; 80]);

  let raw = shared::Bytes::from(vec![b'w'; 80]);
  let mutable: bytes::BytesMut = raw.into();
  assert_eq!(mutable.as_ref(), &[b'w'; 80]);

  let fixed = Utf8Buffer::try_from("fixed").unwrap();
  let bytes: Vec<u8> = fixed.into();
  assert_eq!(bytes, b"fixed");

  let boxed = "p".repeat(80).into_boxed_str();
  let pointer = boxed.as_ptr();
  let text: Utf8Bytes = boxed.into();
  assert_eq!(text.as_str().as_ptr(), pointer);
  let pointer = text.as_str().as_ptr();
  let boxed: Box<str> = text.into();
  assert_eq!(boxed.as_ptr(), pointer);
}

#[cfg(feature = "std")]
#[test]
fn utf8_wrappers_are_os_and_path_references_under_std() {
  let fixed = Utf8Buffer::try_from("relative/path").unwrap();
  let immutable = Utf8Bytes::from("relative/path");
  let mutable = Utf8BytesMut::from("relative/path");

  for value in [
    AsRef::<OsStr>::as_ref(&fixed),
    AsRef::<OsStr>::as_ref(&immutable),
    AsRef::<OsStr>::as_ref(&mutable),
  ] {
    assert_eq!(value, OsStr::new("relative/path"));
  }

  assert_eq!(AsRef::<Path>::as_ref(&fixed), Path::new("relative/path"));
  assert_eq!(
    AsRef::<Path>::as_ref(&immutable),
    Path::new("relative/path")
  );
  assert_eq!(AsRef::<Path>::as_ref(&mutable), Path::new("relative/path"));
}
