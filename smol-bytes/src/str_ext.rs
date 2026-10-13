use core::fmt;

use crate::{Utf8Bytes, Utf8BytesMut};

/// Converts a value to shared or compact UTF-8 bytes using its [`fmt::Display`] implementation.
///
/// [`ToUtf8Bytes::to_utf8bytes`] returns the default shared [`Utf8Bytes`],
/// which retains heap storage after shrinking. [`ToUtf8Bytes::to_compact_utf8bytes`]
/// returns [`crate::compact::Utf8Bytes`], which moves short contents back to
/// inline storage.
///
/// # Examples
///
/// ```
/// use smol_bytes::ToUtf8Bytes;
///
/// assert_eq!(42.to_utf8bytes().as_str(), "42");
/// ```
pub trait ToUtf8Bytes {
  /// Converts this value to the default shared [`Utf8Bytes`] using [`fmt::Display`].
  ///
  /// Shared results retain heap storage when they are later shortened. Use
  /// [`ToUtf8Bytes::to_compact_utf8bytes`] when short results should use inline storage.
  #[must_use = "this returns a new Utf8Bytes without modifying the original"]
  fn to_utf8bytes(&self) -> Utf8Bytes;

  /// Converts this value to compact [`crate::compact::Utf8Bytes`] using [`fmt::Display`].
  ///
  /// Compact results use inline storage when their contents fit. Use
  /// [`ToUtf8Bytes::to_utf8bytes`] when retaining heap storage after shrinking is preferred.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::{ToUtf8Bytes, compact};
  ///
  /// let value: compact::Utf8Bytes = 42.to_compact_utf8bytes();
  /// assert_eq!(value.as_str(), "42");
  /// ```
  #[must_use = "this returns a new compact Utf8Bytes without modifying the original"]
  fn to_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes;
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

  fn to_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes {
    let mut value = Utf8BytesMut::new();
    fmt::Write::write_fmt(&mut value, format_args!("{self}"))
      .expect("a formatting trait implementation returned an error");
    value.freeze_compact()
  }
}

/// [`str`] methods that return default shared or compact UTF-8 bytes.
///
/// Methods ending in `_utf8bytes` return the default shared [`Utf8Bytes`],
/// retaining heap storage after shrinking. Their `_compact_utf8bytes`
/// counterparts return [`crate::compact::Utf8Bytes`], which use inline storage
/// whenever their contents fit.
///
/// This trait is sealed and can only be implemented for [`str`] by this crate.
pub trait StrExt: private::Sealed {
  /// Returns the lowercase equivalent as the default shared [`Utf8Bytes`].
  ///
  /// Shared results retain heap storage when they are later shortened. Use
  /// [`StrExt::to_lowercase_compact_utf8bytes`] when short results should use inline storage.
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

  /// Returns the lowercase equivalent as compact [`crate::compact::Utf8Bytes`].
  ///
  /// Compact results use inline storage when their contents fit. Use
  /// [`StrExt::to_lowercase_utf8bytes`] when retaining heap storage after shrinking is preferred.
  ///
  /// See [`str::to_lowercase`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!("RÜST".to_lowercase_compact_utf8bytes().as_str(), "rüst");
  /// ```
  #[must_use = "this returns a new compact Utf8Bytes without modifying the original"]
  fn to_lowercase_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes;

  /// Returns the uppercase equivalent as the default shared [`Utf8Bytes`].
  ///
  /// Shared results retain heap storage when they are later shortened. Use
  /// [`StrExt::to_uppercase_compact_utf8bytes`] when short results should use inline storage.
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

  /// Returns the uppercase equivalent as compact [`crate::compact::Utf8Bytes`].
  ///
  /// Compact results use inline storage when their contents fit. Use
  /// [`StrExt::to_uppercase_utf8bytes`] when retaining heap storage after shrinking is preferred.
  ///
  /// See [`str::to_uppercase`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!("straße".to_uppercase_compact_utf8bytes().as_str(), "STRASSE");
  /// ```
  #[must_use = "this returns a new compact Utf8Bytes without modifying the original"]
  fn to_uppercase_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes;

  /// Returns the ASCII lowercase equivalent as the default shared [`Utf8Bytes`].
  ///
  /// Shared results retain heap storage when they are later shortened. Use
  /// [`StrExt::to_ascii_lowercase_compact_utf8bytes`] when short results should use inline
  /// storage.
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

  /// Returns the ASCII lowercase equivalent as compact [`crate::compact::Utf8Bytes`].
  ///
  /// Compact results use inline storage when their contents fit. Use
  /// [`StrExt::to_ascii_lowercase_utf8bytes`] when retaining heap storage after shrinking is
  /// preferred.
  ///
  /// See [`str::to_ascii_lowercase`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!("RuSt".to_ascii_lowercase_compact_utf8bytes().as_str(), "rust");
  /// ```
  #[must_use = "this returns a new compact Utf8Bytes without modifying the original"]
  fn to_ascii_lowercase_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes;

  /// Returns the ASCII uppercase equivalent as the default shared [`Utf8Bytes`].
  ///
  /// Shared results retain heap storage when they are later shortened. Use
  /// [`StrExt::to_ascii_uppercase_compact_utf8bytes`] when short results should use inline
  /// storage.
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

  /// Returns the ASCII uppercase equivalent as compact [`crate::compact::Utf8Bytes`].
  ///
  /// Compact results use inline storage when their contents fit. Use
  /// [`StrExt::to_ascii_uppercase_utf8bytes`] when retaining heap storage after shrinking is
  /// preferred.
  ///
  /// See [`str::to_ascii_uppercase`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!("RuSt".to_ascii_uppercase_compact_utf8bytes().as_str(), "RUST");
  /// ```
  #[must_use = "this returns a new compact Utf8Bytes without modifying the original"]
  fn to_ascii_uppercase_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes;

  /// Replaces every match of `from` with `to`, returning default shared [`Utf8Bytes`].
  ///
  /// Shared results retain heap storage when they are later shortened. Use
  /// [`StrExt::replace_compact_utf8bytes`] when short results should use inline storage.
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

  /// Replaces every match of `from` with `to`, returning compact [`crate::compact::Utf8Bytes`].
  ///
  /// Compact results use inline storage when their contents fit. Use
  /// [`StrExt::replace_utf8bytes`] when retaining heap storage after shrinking is preferred.
  ///
  /// See [`str::replace`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!(
  ///   "one one".replace_compact_utf8bytes("one", "two").as_str(),
  ///   "two two",
  /// );
  /// ```
  #[must_use = "this returns a new compact Utf8Bytes without modifying the original"]
  fn replace_compact_utf8bytes(&self, from: &str, to: &str) -> crate::compact::Utf8Bytes;

  /// Replaces at most `count` matches of `from` with `to`, returning default shared
  /// [`Utf8Bytes`].
  ///
  /// Shared results retain heap storage when they are later shortened. Use
  /// [`StrExt::replacen_compact_utf8bytes`] when short results should use inline storage.
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

  /// Replaces at most `count` matches of `from` with `to`, returning compact
  /// [`crate::compact::Utf8Bytes`].
  ///
  /// Compact results use inline storage when their contents fit. Use
  /// [`StrExt::replacen_utf8bytes`] when retaining heap storage after shrinking is preferred.
  ///
  /// See [`str::replacen`].
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::StrExt;
  ///
  /// assert_eq!(
  ///   "one one".replacen_compact_utf8bytes("one", "two", 1).as_str(),
  ///   "two one",
  /// );
  /// ```
  #[must_use = "this returns a new compact Utf8Bytes without modifying the original"]
  fn replacen_compact_utf8bytes(
    &self,
    from: &str,
    to: &str,
    count: usize,
  ) -> crate::compact::Utf8Bytes;
}

impl StrExt for str {
  #[inline]
  fn to_lowercase_utf8bytes(&self) -> Utf8Bytes {
    self.to_lowercase().into()
  }

  #[inline]
  fn to_lowercase_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes {
    self.to_lowercase().into()
  }

  #[inline]
  fn to_uppercase_utf8bytes(&self) -> Utf8Bytes {
    self.to_uppercase().into()
  }

  #[inline]
  fn to_uppercase_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes {
    self.to_uppercase().into()
  }

  #[inline]
  fn to_ascii_lowercase_utf8bytes(&self) -> Utf8Bytes {
    self.to_ascii_lowercase().into()
  }

  #[inline]
  fn to_ascii_lowercase_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes {
    self.to_ascii_lowercase().into()
  }

  #[inline]
  fn to_ascii_uppercase_utf8bytes(&self) -> Utf8Bytes {
    self.to_ascii_uppercase().into()
  }

  #[inline]
  fn to_ascii_uppercase_compact_utf8bytes(&self) -> crate::compact::Utf8Bytes {
    self.to_ascii_uppercase().into()
  }

  #[inline]
  fn replace_utf8bytes(&self, from: &str, to: &str) -> Utf8Bytes {
    self.replace(from, to).into()
  }

  #[inline]
  fn replace_compact_utf8bytes(&self, from: &str, to: &str) -> crate::compact::Utf8Bytes {
    self.replace(from, to).into()
  }

  #[inline]
  fn replacen_utf8bytes(&self, from: &str, to: &str, count: usize) -> Utf8Bytes {
    self.replacen(from, to, count).into()
  }

  #[inline]
  fn replacen_compact_utf8bytes(
    &self,
    from: &str,
    to: &str,
    count: usize,
  ) -> crate::compact::Utf8Bytes {
    self.replacen(from, to, count).into()
  }
}

mod private {
  pub trait Sealed {}

  impl Sealed for str {}
}
