#![warn(rust_2018_idioms)]

//! Coverage of the public error types in `smol_bytes::error`: exact `Display`
//! text, `From` conversions, and the `std::io::Error` bridges. Every error is
//! constructed through the public API wherever a real operation can produce it.

use std::error::Error as _;
#[cfg(feature = "std")]
use std::io;

use smol_bytes::{
  BufTryGetError, Buffer, Bytes, BytesMut, FromBytesError, FromUtf8Error, InvalidIntegerLength,
  OutOfBounds, RangeOutOfBounds, TryGetError, TryGetIntegerError, TryPutError, TryPutIntegerError,
  Utf8Bytes, Utf8BytesMut, Utf8IndexError,
};

// ---------------------------------------------------------------------------
// TryPutError
// ---------------------------------------------------------------------------

#[test]
fn try_put_error_display_and_io() {
  // Real overflow: a fresh Buffer holds 62 bytes, writing 63 fails.
  let big = [0u8; 63];
  let err: TryPutError = Buffer::try_from(&big[..]).unwrap_err();
  assert_eq!(err.requested, 63);
  assert_eq!(err.available, 62);
  assert_eq!(
    err.to_string(),
    "Not enough bytes remaining in buffer to write value (requested 63 but only 62 available)"
  );
  assert!(err.source().is_none());

  #[cfg(feature = "std")]
  {
    // From<TryPutError> for io::Error uses io::Error::other (kind Other).
    let io_err: io::Error = err.into();
    assert_eq!(io_err.kind(), io::ErrorKind::Other);
    assert!(
      io_err
        .to_string()
        .contains("Not enough bytes remaining in buffer to write value")
    );
  }
}

// ---------------------------------------------------------------------------
// InvalidIntegerLength
// ---------------------------------------------------------------------------

#[test]
fn invalid_integer_length_display_from_and_io() {
  let err = InvalidIntegerLength(9);
  assert_eq!(
    err.to_string(),
    "invalid integer length: 9 (must be less or equal to 8)"
  );

  // From<InvalidIntegerLength> for TryPutIntegerError.
  let promoted: TryPutIntegerError = err.into();
  assert_eq!(promoted, TryPutIntegerError::InvalidLength(err));
  assert_eq!(
    promoted.to_string(),
    "invalid integer length: 9 (must be less or equal to 8)"
  );

  #[cfg(feature = "std")]
  {
    // From<InvalidIntegerLength> for io::Error uses InvalidInput.
    let io_err: io::Error = err.into();
    assert_eq!(io_err.kind(), io::ErrorKind::InvalidInput);
    assert!(io_err.to_string().contains("invalid integer length: 9"));
  }
}

// ---------------------------------------------------------------------------
// TryPutIntegerError (both variants)
// ---------------------------------------------------------------------------

#[test]
fn try_put_integer_error_not_enough_space() {
  // 60 bytes already present leaves room for only 2 more; asking for 3 fails.
  let mut buf = Buffer::try_from(&[0u8; 60][..]).unwrap();
  let err = buf.try_put_uint(0x010203, 3).unwrap_err();
  assert_eq!(
    err,
    TryPutIntegerError::NotEnoughSpace(TryPutError {
      requested: 3,
      available: 2,
    })
  );
  // Display forwards to the inner TryPutError.
  assert_eq!(
    err.to_string(),
    "Not enough bytes remaining in buffer to write value (requested 3 but only 2 available)"
  );

  #[cfg(feature = "std")]
  {
    let io_err: io::Error = err.into();
    assert_eq!(io_err.kind(), io::ErrorKind::Other);
  }
}

#[test]
fn try_put_integer_error_invalid_length() {
  let mut buf = Buffer::new();
  let err = buf.try_put_uint(0, 9).unwrap_err();
  assert_eq!(
    err,
    TryPutIntegerError::InvalidLength(InvalidIntegerLength(9))
  );
  // Display forwards to the inner InvalidIntegerLength.
  assert_eq!(
    err.to_string(),
    "invalid integer length: 9 (must be less or equal to 8)"
  );
}

// ---------------------------------------------------------------------------
// OutOfBounds
// ---------------------------------------------------------------------------

#[test]
fn out_of_bounds_display_and_io() {
  // try_advance past the end of a 3-byte buffer.
  let mut buf = Buffer::try_from(&b"abc"[..]).unwrap();
  let err: OutOfBounds = buf.try_advance(5).unwrap_err();
  assert_eq!(err, OutOfBounds::new(5, 3));
  assert_eq!(
    err.to_string(),
    "index out of bounds: requested 5 but only 3 available"
  );

  // try_split_off also surfaces OutOfBounds.
  let mut buf = Buffer::try_from(&b"abc"[..]).unwrap();
  assert_eq!(buf.try_split_off(10).unwrap_err(), OutOfBounds::new(10, 3));

  #[cfg(feature = "std")]
  {
    let io_err: io::Error = err.into();
    assert_eq!(io_err.kind(), io::ErrorKind::InvalidInput);
    assert!(
      io_err
        .to_string()
        .contains("index out of bounds: requested 5")
    );
  }
}

// ---------------------------------------------------------------------------
// RangeOutOfBounds
// ---------------------------------------------------------------------------

#[test]
fn range_out_of_bounds_reversed_and_oob() {
  let buf = Buffer::try_from(&b"hello"[..]).unwrap();

  // Reversed range: start > end (built from values to express the intent
  // without tripping the reversed-range lint on a literal range).
  let (start, end) = (3usize, 2usize);
  let err: RangeOutOfBounds = buf.try_slice(start..end).unwrap_err();
  assert_eq!(err, RangeOutOfBounds::new(3, 2, 5));
  assert_eq!(
    err.to_string(),
    "range out of bounds: requested 3..2 but only 5 available"
  );

  // End beyond the length.
  let err2 = buf.try_slice(0..10).unwrap_err();
  assert_eq!(err2, RangeOutOfBounds::new(0, 10, 5));

  #[cfg(feature = "std")]
  {
    let io_err: io::Error = err.into();
    assert_eq!(io_err.kind(), io::ErrorKind::InvalidInput);
    assert!(
      io_err
        .to_string()
        .contains("range out of bounds: requested 3..2")
    );
  }
}

// ---------------------------------------------------------------------------
// Utf8IndexError (all variants)
// ---------------------------------------------------------------------------

#[test]
fn utf8_error_invalid_char_boundary() {
  // "café" == [c, a, f, 0xC3, 0xA9]; byte index 4 splits the 'é'.
  let mut s = Utf8BytesMut::from("café");
  let err: Utf8IndexError = s.try_split_to(4).unwrap_err();
  assert_eq!(err, Utf8IndexError::InvalidCharBoundary { index: 4 });
  assert_eq!(
    err.to_string(),
    "index 4 does not lie on a UTF-8 character boundary"
  );

  #[cfg(feature = "std")]
  {
    let io_err: io::Error = err.into();
    assert_eq!(io_err.kind(), io::ErrorKind::InvalidInput);
    assert!(
      io_err
        .to_string()
        .contains("does not lie on a UTF-8 character boundary")
    );
  }
}

#[test]
fn utf8_index_error_preserves_index_and_range_context() {
  let mut s = Utf8Bytes::from("hi");
  let index_error: Utf8IndexError = s.try_split_to(10).unwrap_err();
  assert_eq!(
    index_error,
    Utf8IndexError::IndexOutOfBounds(OutOfBounds::new(10, 2))
  );
  assert!(index_error.source().is_some());

  let (start, end) = (3, 2);
  let range_error: Utf8IndexError = s.try_slice(start..end).unwrap_err();
  assert_eq!(
    range_error,
    Utf8IndexError::RangeOutOfBounds(RangeOutOfBounds::new(3, 2, 2))
  );
  assert!(range_error.source().is_some());
}

// ---------------------------------------------------------------------------
// TryGetError and TryGetIntegerError
// ---------------------------------------------------------------------------

#[test]
fn variable_width_read_errors_are_typed_and_atomic() {
  let mut buffer = Buffer::from(*b"abc");
  let before = buffer.as_slice().to_vec();
  assert_eq!(
    buffer.try_get_uint(9),
    Err(TryGetIntegerError::InvalidLength(InvalidIntegerLength(9)))
  );
  assert_eq!(buffer.as_slice(), before);

  assert_eq!(
    buffer.try_get_int_le(4),
    Err(TryGetIntegerError::NotEnoughBytes(TryGetError {
      requested: 4,
      available: 3,
    }))
  );
  assert_eq!(buffer.as_slice(), before);

  for result in [
    buffer.try_get_uint(9).map(|_| ()),
    buffer.try_get_uint_le(9).map(|_| ()),
    buffer.try_get_uint_ne(9).map(|_| ()),
    buffer.try_get_int(9).map(|_| ()),
    buffer.try_get_int_le(9).map(|_| ()),
    buffer.try_get_int_ne(9).map(|_| ()),
  ] {
    assert_eq!(
      result,
      Err(TryGetIntegerError::InvalidLength(InvalidIntegerLength(9)))
    );
  }
  assert_eq!(buffer.as_slice(), before);

  let crate_error = TryGetError {
    requested: 4,
    available: 3,
  };
  let upstream: BufTryGetError = crate_error.into();
  assert_eq!(TryGetError::from(upstream), crate_error);

  let mut trait_buffer = Buffer::from(*b"abc");
  let upstream: BufTryGetError = bytes::Buf::try_get_uint(&mut trait_buffer, 4).unwrap_err();
  assert_eq!(TryGetError::from(upstream), crate_error);
}

// ---------------------------------------------------------------------------
// FromUtf8Error
// ---------------------------------------------------------------------------

#[test]
fn owned_utf8_errors_recover_original_storage_without_copying() {
  let mut raw = BytesMut::from(vec![b'x'; 80]);
  raw.as_mut_slice()[79] = 0xff;
  let pointer = raw.as_slice().as_ptr();
  let error: FromUtf8Error<BytesMut> = match Utf8BytesMut::from_utf8(raw) {
    Err(error) => error,
    Ok(_) => panic!("invalid bytes unexpectedly validated"),
  };
  assert_eq!(error.as_bytes().as_ptr(), pointer);
  assert!(error.utf8_error().valid_up_to() < error.as_bytes().len());
  assert!(error.source().is_some());
  let raw = error.into_bytes();
  assert_eq!(raw.as_slice().as_ptr(), pointer);
  assert_eq!(raw.as_slice()[79], 0xff);

  let mut raw_bytes = vec![b'x'; 80];
  raw_bytes[79] = 0xff;
  let mut raw = Bytes::from(raw_bytes);
  let _ = raw.split_to(5);
  let pointer = raw.as_slice().as_ptr();
  let error = match Utf8Bytes::from_utf8(raw) {
    Err(error) => error,
    Ok(_) => panic!("invalid bytes unexpectedly validated"),
  };
  assert_eq!(error.as_bytes().as_ptr(), pointer);
  assert_eq!(error.into_bytes().as_slice().as_ptr(), pointer);

  let buffer = Buffer::try_from(&[b'a', 0xff][..]).unwrap();
  let error = match smol_bytes::Utf8Buffer::from_utf8(buffer) {
    Err(error) => error,
    Ok(_) => panic!("invalid bytes unexpectedly validated"),
  };
  assert_eq!(error.as_bytes(), b"a\xff");
  assert_eq!(error.into_bytes().as_slice(), b"a\xff");
}

#[test]
fn raw_bytes_try_into_mut_returns_the_same_strategy_on_failure() {
  let original = Bytes::from_static(&[b'x'; 80]);
  let recovered: Bytes = match original.try_into_mut() {
    Ok(_) => panic!("static bytes unexpectedly became mutable without a copy"),
    Err(bytes) => bytes,
  };
  assert_eq!(recovered.as_slice(), &[b'x'; 80]);
}

// ---------------------------------------------------------------------------
// FromBytesError (both variants) + From conversions
// ---------------------------------------------------------------------------

#[test]
fn from_bytes_error_invalid_utf8() {
  // 0xFF is never valid UTF-8. Build the input at runtime so the conversion is
  // exercised as a real fallible operation (not folded to a compile-time error).
  let raw: Vec<u8> = vec![0xFF, 0xFE];
  let err: FromBytesError = smol_bytes::Utf8Buffer::try_from(raw.as_slice()).unwrap_err();
  assert!(matches!(err, FromBytesError::InvalidUtf8(_)));
  let text = err.to_string();
  assert!(text.starts_with("invalid UTF-8: "), "got: {text}");

  // The wrapped std Utf8Error's own Display is included.
  let inner = core::str::from_utf8(raw.as_slice()).unwrap_err();
  assert!(text.contains(&inner.to_string()));

  // FromBytesError does not chain its wrapped error as a source.
  assert!(err.source().is_none());

  // From<core::str::Utf8Error> for FromBytesError.
  let direct: FromBytesError = inner.into();
  assert_eq!(direct, err);

  #[cfg(feature = "std")]
  {
    let io_err: io::Error = err.into();
    assert_eq!(io_err.kind(), io::ErrorKind::InvalidInput);
    assert!(io_err.to_string().contains("invalid UTF-8"));
  }
}

#[test]
fn from_bytes_error_too_large() {
  // 63 bytes of valid ASCII exceeds the 62-byte inline capacity.
  let long = [b'a'; 63];
  let err: FromBytesError = smol_bytes::Utf8Buffer::try_from(&long[..]).unwrap_err();
  assert!(matches!(err, FromBytesError::TooLarge(_)));
  // Display forwards to the inner TryPutError verbatim.
  assert_eq!(
    err.to_string(),
    "Not enough bytes remaining in buffer to write value (requested 63 but only 62 available)"
  );
  assert!(err.source().is_none());

  // From<TryPutError> for FromBytesError.
  let put_err = TryPutError {
    requested: 63,
    available: 62,
  };
  let converted: FromBytesError = put_err.into();
  assert_eq!(converted, err);

  #[cfg(feature = "std")]
  {
    let io_err: io::Error = err.into();
    assert_eq!(io_err.kind(), io::ErrorKind::InvalidInput);
  }
}
