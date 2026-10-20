use core::ops::{Bound, RangeBounds};

/// The error returned by [`bytes::Buf`] fallible readers when allocation is
/// available.
///
/// This is re-exported separately because [`TryGetError`] is the crate-owned
/// error used by [`crate::Buffer`]'s inherent API.
#[cfg(any(feature = "std", feature = "alloc"))]
pub use bytes::TryGetError as BufTryGetError;

/// Error type for the `try_get_` methods of [`crate::Buffer`].
/// Indicates that there were not enough remaining
/// bytes in the buffer while attempting
/// to get a value from a [`crate::Buffer`] with one
/// of the `try_get_` methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TryGetError {
  /// The number of bytes necessary to get the value
  pub requested: usize,

  /// The number of bytes available in the buffer
  pub available: usize,
}

impl core::fmt::Display for TryGetError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    write!(
      f,
      "Not enough bytes remaining in buffer to read value (requested {} but only {} available)",
      self.requested, self.available
    )
  }
}

impl core::error::Error for TryGetError {}

#[cfg(any(feature = "std", feature = "alloc"))]
impl From<bytes::TryGetError> for TryGetError {
  fn from(value: bytes::TryGetError) -> Self {
    Self {
      requested: value.requested,
      available: value.available,
    }
  }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl From<TryGetError> for bytes::TryGetError {
  fn from(value: TryGetError) -> Self {
    Self {
      requested: value.requested,
      available: value.available,
    }
  }
}

#[cfg(feature = "std")]
impl From<TryGetError> for std::io::Error {
  fn from(value: TryGetError) -> Self {
    std::io::Error::other(value)
  }
}

/// Error type for the `try_put_` methods of [`crate::Buffer`].
/// Indicates that there were not enough remaining
/// capacity in the buffer while attempting
/// to put a value to a [`crate::Buffer`] with one
/// of the `try_put_` methods.
#[derive(Debug, PartialEq, Eq)]
pub struct TryPutError {
  /// The number of bytes necessary to put the value
  pub requested: usize,

  /// The number of bytes available in the buffer
  pub available: usize,
}

impl core::fmt::Display for TryPutError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    write!(
      f,
      "Not enough bytes remaining in buffer to write value (requested {} but only {} available)",
      self.requested, self.available
    )
  }
}

impl core::error::Error for TryPutError {}

#[cfg(feature = "std")]
impl From<TryPutError> for std::io::Error {
  fn from(value: TryPutError) -> Self {
    std::io::Error::other(value)
  }
}

/// Error type indicating an invalid integer length was specified.
/// This is used when attempting to read or write
/// an integer with an invalid number of bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InvalidIntegerLength(pub usize);

impl core::fmt::Display for InvalidIntegerLength {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    write!(
      f,
      "invalid integer length: {} (must be less or equal to 8)",
      self.0
    )
  }
}

impl core::error::Error for InvalidIntegerLength {}

#[cfg(feature = "std")]
impl From<InvalidIntegerLength> for std::io::Error {
  fn from(value: InvalidIntegerLength) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, value)
  }
}

impl From<InvalidIntegerLength> for TryPutIntegerError {
  fn from(value: InvalidIntegerLength) -> Self {
    TryPutIntegerError::InvalidLength(value)
  }
}

/// Error returned by checked variable-width integer writers.
///
/// Unlike [`TryPutIntegerError`], this error distinguishes values that cannot
/// be represented in the requested width from values that merely need more
/// writable capacity.
#[derive(Debug, PartialEq, Eq)]
pub enum TryPutIntegerCheckedError {
  /// The requested integer width was outside the supported `0..=8` range.
  InvalidLength(InvalidIntegerLength),
  /// The value cannot be represented in the requested width.
  OutOfRange {
    /// The value supplied by the caller.
    value: i128,
    /// The smallest value representable by the requested width.
    min: i128,
    /// The largest value representable by the requested width.
    max: i128,
    /// The requested width in bytes.
    nbytes: usize,
  },
  /// There was not enough writable capacity for the checked value.
  NotEnoughSpace(TryPutError),
}

impl core::fmt::Display for TryPutIntegerCheckedError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    match self {
      Self::InvalidLength(error) => error.fmt(f),
      Self::OutOfRange {
        value,
        min,
        max,
        nbytes,
      } => write!(
        f,
        "value {value} is out of range for a {nbytes}-byte integer ({min}..={max})"
      ),
      Self::NotEnoughSpace(error) => error.fmt(f),
    }
  }
}

impl core::error::Error for TryPutIntegerCheckedError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    match self {
      Self::InvalidLength(error) => Some(error),
      Self::OutOfRange { .. } => None,
      Self::NotEnoughSpace(error) => Some(error),
    }
  }
}

impl From<InvalidIntegerLength> for TryPutIntegerCheckedError {
  fn from(value: InvalidIntegerLength) -> Self {
    Self::InvalidLength(value)
  }
}

impl From<TryPutError> for TryPutIntegerCheckedError {
  fn from(value: TryPutError) -> Self {
    Self::NotEnoughSpace(value)
  }
}

/// Error returned by variable-width integer readers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TryGetIntegerError {
  /// Not enough bytes remained to read the requested value.
  NotEnoughBytes(TryGetError),
  /// The requested integer width was outside the supported `0..=8` range.
  InvalidLength(InvalidIntegerLength),
}

impl core::fmt::Display for TryGetIntegerError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    match self {
      Self::NotEnoughBytes(error) => error.fmt(f),
      Self::InvalidLength(error) => error.fmt(f),
    }
  }
}

impl core::error::Error for TryGetIntegerError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    match self {
      Self::NotEnoughBytes(error) => Some(error),
      Self::InvalidLength(error) => Some(error),
    }
  }
}

impl From<TryGetError> for TryGetIntegerError {
  fn from(value: TryGetError) -> Self {
    Self::NotEnoughBytes(value)
  }
}

impl From<InvalidIntegerLength> for TryGetIntegerError {
  fn from(value: InvalidIntegerLength) -> Self {
    Self::InvalidLength(value)
  }
}

#[cfg(feature = "std")]
impl From<TryGetIntegerError> for std::io::Error {
  fn from(value: TryGetIntegerError) -> Self {
    std::io::Error::other(value)
  }
}

/// Error type for the `try_put_` methods of [`crate::Buffer`].
/// Indicates that there were not enough remaining
/// capacity in the buffer while attempting
/// to put a value to a [`crate::Buffer`] with one
/// of the `try_put_` methods.
#[derive(Debug, PartialEq, Eq)]
pub enum TryPutIntegerError {
  /// Not enough space to write all bytes.
  NotEnoughSpace(TryPutError),
  /// Invalid number of bytes specified.
  InvalidLength(InvalidIntegerLength),
}

impl core::fmt::Display for TryPutIntegerError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    match self {
      Self::NotEnoughSpace(err) => write!(f, "{}", err),
      Self::InvalidLength(len) => write!(f, "{}", len),
    }
  }
}

impl core::error::Error for TryPutIntegerError {}

#[cfg(feature = "std")]
impl From<TryPutIntegerError> for std::io::Error {
  fn from(value: TryPutIntegerError) -> Self {
    std::io::Error::other(value)
  }
}
/// Error type indicating an index or range is out of bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutOfBounds {
  /// The requested index or size
  pub requested: usize,
  /// The available length or capacity
  pub available: usize,
}

impl OutOfBounds {
  /// Creates a new `OutOfBounds` error.
  #[inline]
  pub const fn new(requested: usize, available: usize) -> Self {
    Self {
      requested,
      available,
    }
  }
}

impl core::fmt::Display for OutOfBounds {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    write!(
      f,
      "index out of bounds: requested {} but only {} available",
      self.requested, self.available
    )
  }
}

impl core::error::Error for OutOfBounds {}

#[cfg(feature = "std")]
impl From<OutOfBounds> for std::io::Error {
  fn from(value: OutOfBounds) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, value)
  }
}

/// Error type indicating an index or range is out of bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RangeOutOfBounds {
  /// The start index of the range
  pub start: usize,
  /// The end index of the requested range
  pub end: usize,
  /// The available length or capacity
  pub available: usize,
}

impl RangeOutOfBounds {
  /// Creates a new `RangeOutOfBounds` error.
  #[inline]
  pub const fn new(start: usize, end: usize, available: usize) -> Self {
    Self {
      start,
      end,
      available,
    }
  }
}

impl core::fmt::Display for RangeOutOfBounds {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    write!(
      f,
      "range out of bounds: requested {}..{} but only {} available",
      self.start, self.end, self.available
    )
  }
}

impl core::error::Error for RangeOutOfBounds {}

/// Error returned by raw byte editing operations.
#[derive(Debug, PartialEq, Eq)]
pub enum ByteEditError {
  /// The requested byte range was invalid for the visible buffer contents.
  Range(RangeOutOfBounds),
  /// The replacement cannot fit in a fixed-capacity buffer.
  Capacity(TryPutError),
}

impl core::fmt::Display for ByteEditError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    match self {
      Self::Range(error) => error.fmt(f),
      Self::Capacity(error) => error.fmt(f),
    }
  }
}

impl core::error::Error for ByteEditError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    match self {
      Self::Range(error) => Some(error),
      Self::Capacity(error) => Some(error),
    }
  }
}

impl From<RangeOutOfBounds> for ByteEditError {
  fn from(value: RangeOutOfBounds) -> Self {
    Self::Range(value)
  }
}

impl From<TryPutError> for ByteEditError {
  fn from(value: TryPutError) -> Self {
    Self::Capacity(value)
  }
}

/// Normalizes a range against `len` without panicking on reversed or
/// overflowing bounds.
pub(crate) fn normalize_range(
  range: impl RangeBounds<usize>,
  len: usize,
) -> Result<(usize, usize), RangeOutOfBounds> {
  let start = match range.start_bound() {
    Bound::Included(&n) => n,
    Bound::Excluded(&n) => n
      .checked_add(1)
      .ok_or_else(|| RangeOutOfBounds::new(n, n, len))?,
    Bound::Unbounded => 0,
  };

  let end = match range.end_bound() {
    Bound::Included(&n) => n
      .checked_add(1)
      .ok_or_else(|| RangeOutOfBounds::new(start, n, len))?,
    Bound::Excluded(&n) => n,
    Bound::Unbounded => len,
  };

  if start > end || end > len {
    return Err(RangeOutOfBounds::new(start, end, len));
  }

  Ok((start, end))
}

#[cfg(feature = "std")]
impl From<RangeOutOfBounds> for std::io::Error {
  fn from(value: RangeOutOfBounds) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, value)
  }
}

/// Error type for UTF-8 index and range operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Utf8IndexError {
  /// The index does not lie on a UTF-8 character boundary.
  InvalidCharBoundary {
    /// The index that was not on a character boundary
    index: usize,
  },
  /// A single index was outside the visible bounds.
  IndexOutOfBounds(OutOfBounds),
  /// A range was reversed, overflowing, or outside the visible bounds.
  RangeOutOfBounds(RangeOutOfBounds),
}

impl core::fmt::Display for Utf8IndexError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    match self {
      Self::InvalidCharBoundary { index } => {
        write!(
          f,
          "index {} does not lie on a UTF-8 character boundary",
          index
        )
      }
      Self::IndexOutOfBounds(error) => error.fmt(f),
      Self::RangeOutOfBounds(error) => error.fmt(f),
    }
  }
}

impl core::error::Error for Utf8IndexError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    match self {
      Self::InvalidCharBoundary { .. } => None,
      Self::IndexOutOfBounds(error) => Some(error),
      Self::RangeOutOfBounds(error) => Some(error),
    }
  }
}

impl From<OutOfBounds> for Utf8IndexError {
  fn from(value: OutOfBounds) -> Self {
    Self::IndexOutOfBounds(value)
  }
}

impl From<RangeOutOfBounds> for Utf8IndexError {
  fn from(value: RangeOutOfBounds) -> Self {
    Self::RangeOutOfBounds(value)
  }
}

/// Error type for fixed-capacity UTF-8 editing operations.
#[derive(Debug, PartialEq, Eq)]
pub enum Utf8EditError {
  /// The requested index or range is invalid.
  Index(Utf8IndexError),
  /// The replacement cannot fit in the fixed-capacity buffer.
  Capacity(TryPutError),
}

impl core::fmt::Display for Utf8EditError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    match self {
      Self::Index(error) => error.fmt(f),
      Self::Capacity(error) => error.fmt(f),
    }
  }
}

impl core::error::Error for Utf8EditError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    match self {
      Self::Index(error) => Some(error),
      Self::Capacity(error) => Some(error),
    }
  }
}

impl From<Utf8IndexError> for Utf8EditError {
  fn from(value: Utf8IndexError) -> Self {
    Self::Index(value)
  }
}

impl From<TryPutError> for Utf8EditError {
  fn from(value: TryPutError) -> Self {
    Self::Capacity(value)
  }
}

/// Error returned by fallible formatting into a fixed UTF-8 buffer.
#[derive(Debug, PartialEq, Eq)]
pub enum Utf8FormatError {
  /// Formatting exceeded the fixed buffer's remaining capacity.
  Capacity(TryPutError),
  /// The formatter returned an error unrelated to capacity.
  Formatter(core::fmt::Error),
}

impl core::fmt::Display for Utf8FormatError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    match self {
      Self::Capacity(error) => error.fmt(f),
      Self::Formatter(error) => error.fmt(f),
    }
  }
}

impl core::error::Error for Utf8FormatError {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    match self {
      Self::Capacity(error) => Some(error),
      Self::Formatter(error) => Some(error),
    }
  }
}

impl From<TryPutError> for Utf8FormatError {
  fn from(value: TryPutError) -> Self {
    Self::Capacity(value)
  }
}

impl From<core::fmt::Error> for Utf8FormatError {
  fn from(value: core::fmt::Error) -> Self {
    Self::Formatter(value)
  }
}

#[cfg(feature = "std")]
impl From<Utf8IndexError> for std::io::Error {
  fn from(value: Utf8IndexError) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, value)
  }
}

/// An owned byte container that failed UTF-8 validation.
///
/// The original bytes are retained so callers can inspect or recover the
/// exact input without copying or changing its storage strategy.
#[derive(Debug)]
pub struct FromUtf8Error<B> {
  bytes: B,
  error: core::str::Utf8Error,
}

impl<B> FromUtf8Error<B> {
  #[inline]
  pub(crate) const fn new(bytes: B, error: core::str::Utf8Error) -> Self {
    Self { bytes, error }
  }

  /// Returns the UTF-8 validation error.
  #[inline]
  pub const fn utf8_error(&self) -> core::str::Utf8Error {
    self.error
  }

  /// Recovers the original bytes.
  #[inline]
  pub fn into_bytes(self) -> B {
    self.bytes
  }
}

impl<B> FromUtf8Error<B>
where
  B: AsRef<[u8]>,
{
  /// Returns the original bytes without consuming the error.
  #[inline]
  pub fn as_bytes(&self) -> &[u8] {
    self.bytes.as_ref()
  }
}

impl<B> core::fmt::Display for FromUtf8Error<B> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    write!(f, "invalid UTF-8: {}", self.error)
  }
}

impl<B: core::fmt::Debug> core::error::Error for FromUtf8Error<B> {
  fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
    Some(&self.error)
  }
}

#[cfg(feature = "std")]
impl<B: core::fmt::Debug + Send + Sync + 'static> From<FromUtf8Error<B>> for std::io::Error {
  fn from(value: FromUtf8Error<B>) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidData, value)
  }
}

/// Error type for converting a raw byte slice into a fixed-capacity
/// UTF-8 buffer. Combines UTF-8 validation and capacity errors.
#[derive(Debug, PartialEq, Eq)]
pub enum FromBytesError {
  /// The bytes are not valid UTF-8.
  InvalidUtf8(core::str::Utf8Error),
  /// The bytes do not fit in the buffer's capacity.
  TooLarge(TryPutError),
}

impl core::fmt::Display for FromBytesError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    match self {
      Self::InvalidUtf8(e) => write!(f, "invalid UTF-8: {}", e),
      Self::TooLarge(e) => write!(f, "{}", e),
    }
  }
}

impl core::error::Error for FromBytesError {}

impl From<core::str::Utf8Error> for FromBytesError {
  fn from(e: core::str::Utf8Error) -> Self {
    Self::InvalidUtf8(e)
  }
}

impl From<TryPutError> for FromBytesError {
  fn from(e: TryPutError) -> Self {
    Self::TooLarge(e)
  }
}

#[cfg(feature = "std")]
impl From<FromBytesError> for std::io::Error {
  fn from(value: FromBytesError) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, value)
  }
}
