#[cfg(any(feature = "std", feature = "alloc"))]
pub use bytes::TryGetError;

/// Error type for the `try_get_` methods of [`Buffer`].
/// Indicates that there were not enough remaining
/// bytes in the buffer while attempting
/// to get a value from a [`Buffer`] with one
/// of the `try_get_` methods.
#[derive(Debug, PartialEq, Eq)]
#[cfg(not(any(feature = "std", feature = "alloc")))]
pub struct TryGetError {
  /// The number of bytes necessary to get the value
  pub requested: usize,

  /// The number of bytes available in the buffer
  pub available: usize,
}

#[cfg(not(any(feature = "std", feature = "alloc")))]
const _: () = {
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
};

/// Error type for the `try_put_` methods of [`Buffer`].
/// Indicates that there were not enough remaining
/// capacity in the buffer while attempting
/// to put a value to a [`Buffer`] with one
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

/// Error type for the `try_put_` methods of [`Buffer`].
/// Indicates that there were not enough remaining
/// capacity in the buffer while attempting
/// to put a value to a [`Buffer`] with one
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[cfg(feature = "std")]
impl From<RangeOutOfBounds> for std::io::Error {
  fn from(value: RangeOutOfBounds) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, value)
  }
}

/// Error type for UTF-8 operations that indicates an invalid character boundary
/// or out of bounds access.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Utf8Error {
  /// The index does not lie on a UTF-8 character boundary.
  InvalidCharBoundary {
    /// The index that was not on a character boundary
    at: usize,
  },
  /// The index or range is out of bounds.
  OutOfBounds {
    /// The requested index
    at: usize,
    /// The length of the buffer
    len: usize,
  },
}

impl core::fmt::Display for Utf8Error {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
    match self {
      Self::InvalidCharBoundary { at } => {
        write!(f, "index {} does not lie on a UTF-8 character boundary", at)
      }
      Self::OutOfBounds { at, len } => {
        write!(f, "index {} out of bounds: length is {}", at, len)
      }
    }
  }
}

impl core::error::Error for Utf8Error {}

#[cfg(feature = "std")]
impl From<Utf8Error> for std::io::Error {
  fn from(value: Utf8Error) -> Self {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, value)
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
