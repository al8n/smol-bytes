use core::fmt;

use crate::{Utf8Bytes, Utf8BytesMut};

/// Converts a value to [`Utf8Bytes`] using its [`fmt::Display`] implementation.
///
/// This is comparable to `ToString::to_string`, but returns an immutable
/// shared [`Utf8Bytes`] instead of a [`String`](std::string::String).
///
/// # Examples
///
/// ```
/// use smol_bytes::ToUtf8Bytes;
///
/// assert_eq!(42.to_utf8bytes().as_str(), "42");
/// ```
pub trait ToUtf8Bytes {
  /// Converts this value to [`Utf8Bytes`] using its [`fmt::Display`] implementation.
  #[must_use = "this returns a new Utf8Bytes without modifying the original"]
  fn to_utf8bytes(&self) -> Utf8Bytes;
}

impl<T> ToUtf8Bytes for T
where
  T: fmt::Display + ?Sized,
{
  fn to_utf8bytes(&self) -> Utf8Bytes {
    let mut value = Utf8BytesMut::new();
    fmt::Write::write_fmt(&mut value, format_args!("{self}"))
      .expect("a formatting trait implementation returned an error");
    value.freeze_shared()
  }
}

/// [`str`] methods that return [`Utf8Bytes`].
///
/// This trait is sealed and can only be implemented for [`str`] by this crate.
pub trait StrExt: private::Sealed {
  /// Returns the lowercase equivalent of this string slice as [`Utf8Bytes`].
  ///
  /// See [`str::to_lowercase`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!("RÜST".to_lowercase_utf8bytes().as_str(), "rüst");
  /// ```
  #[must_use = "this returns a new Utf8Bytes without modifying the original"]
  fn to_lowercase_utf8bytes(&self) -> Utf8Bytes;

  /// Returns the uppercase equivalent of this string slice as [`Utf8Bytes`].
  ///
  /// See [`str::to_uppercase`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!("straße".to_uppercase_utf8bytes().as_str(), "STRASSE");
  /// ```
  #[must_use = "this returns a new Utf8Bytes without modifying the original"]
  fn to_uppercase_utf8bytes(&self) -> Utf8Bytes;

  /// Returns the ASCII lowercase equivalent of this string slice as [`Utf8Bytes`].
  ///
  /// See [`str::to_ascii_lowercase`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!("RuSt".to_ascii_lowercase_utf8bytes().as_str(), "rust");
  /// ```
  #[must_use = "this returns a new Utf8Bytes without modifying the original"]
  fn to_ascii_lowercase_utf8bytes(&self) -> Utf8Bytes;

  /// Returns the ASCII uppercase equivalent of this string slice as [`Utf8Bytes`].
  ///
  /// See [`str::to_ascii_uppercase`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!("RuSt".to_ascii_uppercase_utf8bytes().as_str(), "RUST");
  /// ```
  #[must_use = "this returns a new Utf8Bytes without modifying the original"]
  fn to_ascii_uppercase_utf8bytes(&self) -> Utf8Bytes;

  /// Replaces every match of `from` with `to`, returning [`Utf8Bytes`].
  ///
  /// See [`str::replace`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!("one one".replace_utf8bytes("one", "two").as_str(), "two two");
  /// ```
  #[must_use = "this returns a new Utf8Bytes without modifying the original"]
  fn replace_utf8bytes(&self, from: &str, to: &str) -> Utf8Bytes;

  /// Replaces at most `count` matches of `from` with `to`, returning [`Utf8Bytes`].
  ///
  /// See [`str::replacen`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!(
  ///   "one one".replacen_utf8bytes("one", "two", 1).as_str(),
  ///   "two one",
  /// );
  /// ```
  #[must_use = "this returns a new Utf8Bytes without modifying the original"]
  fn replacen_utf8bytes(&self, from: &str, to: &str, count: usize) -> Utf8Bytes;
}

impl StrExt for str {
  #[inline]
  fn to_lowercase_utf8bytes(&self) -> Utf8Bytes {
    self.to_lowercase().into()
  }

  #[inline]
  fn to_uppercase_utf8bytes(&self) -> Utf8Bytes {
    self.to_uppercase().into()
  }

  #[inline]
  fn to_ascii_lowercase_utf8bytes(&self) -> Utf8Bytes {
    self.to_ascii_lowercase().into()
  }

  #[inline]
  fn to_ascii_uppercase_utf8bytes(&self) -> Utf8Bytes {
    self.to_ascii_uppercase().into()
  }

  #[inline]
  fn replace_utf8bytes(&self, from: &str, to: &str) -> Utf8Bytes {
    self.replace(from, to).into()
  }

  #[inline]
  fn replacen_utf8bytes(&self, from: &str, to: &str, count: usize) -> Utf8Bytes {
    self.replacen(from, to, count).into()
  }
}

mod private {
  pub trait Sealed {}

  impl Sealed for str {}
}
