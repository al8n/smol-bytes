//! [`sqlx`](https://docs.rs/sqlx) `Type` / `Encode` / `Decode` bindings.
//!
//! ## Shape
//!
//! One impl trio per type, generic over `DB: Database`, delegating to the
//! standard-library type the driver already supports — `[u8]` / `Vec<u8>` /
//! `&[u8]` for the byte types, `str` / `String` / `&str` for the UTF-8 ones.
//! Every backend that supports those comes along for free; PostgreSQL, MySQL
//! and SQLite all do. This mirrors `sqlx-core`'s own `bstr` integration.
//!
//! ## Why there is no per-backend specialisation
//!
//! The obvious ambition is for a `Decode` that takes an owned or shareable
//! buffer off the row rather than copying out of it. **No backend permits
//! that from third-party code**, checked against sqlx 0.9.0 rather than
//! assumed:
//!
//! - `PgValueRef` does hold a `&Bytes` pointing at the row buffer, but that
//!   field is `pub(crate)`; the only public accessors are `as_bytes()` and
//!   `as_str()`, both of which yield a borrow.
//! - `MySqlValueRef` is stricter still — even `as_bytes()` is `pub(crate)`,
//!   so a foreign `Decode` impl has no way to reach the bytes except through
//!   another `Decode` impl. `ValueRef::to_owned` does slice the row's `Bytes`
//!   without copying, but it returns a `MySqlValue` whose payload is private.
//! - `SqliteValueRef` wraps a `sqlite3_value` owned by SQLite's C allocator.
//!   There is no Rust-side buffer to share at all, `blob_owned()` is a
//!   `to_vec()`, and the handle is deliberately `!Send + !Sync`.
//!
//! So exactly one copy out of the row is unavoidable for every type on every
//! backend, and the delegating shape is not merely convenient — it is the
//! only shape available. What *is* worth choosing carefully is the delegate,
//! and what happens after the copy; see below.
//!
//! ## Copying, per type and direction
//!
//! | Type | `decode` | `encode_by_ref` |
//! | --- | --- | --- |
//! | [`shared::Bytes`](crate::shared::Bytes) | one copy row → `Vec<u8>`, then the `Vec`'s allocation is **moved** into the reference-counted representation above 62 bytes | one copy into the argument buffer |
//! | [`compact::Bytes`](crate::compact::Bytes) | same, except at or below 62 bytes the `Vec` is copied inline and released, by construction | one copy into the argument buffer |
//! | [`Buffer`](crate::Buffer) | one copy row → `Vec<u8>`, then a second inline copy; the `Vec` is discarded | one copy into the argument buffer |
//! | [`shared::Utf8Bytes`](crate::shared::Utf8Bytes) / [`compact::Utf8Bytes`](crate::compact::Utf8Bytes) | one copy row → `String`, UTF-8 validated by the driver, then the allocation is **moved** in above 62 bytes | one copy into the argument buffer |
//! | [`Utf8Buffer`](crate::Utf8Buffer) | one copy row → `String`, then a second inline copy | one copy into the argument buffer |
//!
//! The wasted intermediate for the two inline types is the price of decoding
//! through `Vec<u8>` / `String` instead of `&[u8]` / `&str`. That price is
//! worth paying: on PostgreSQL `&[u8]` refuses to decode `BYTEA` in a simple
//! (unprepared) query outright, and on SQLite decoding a borrow marks the
//! value handle as borrowed so that later decodes of it fail. The owned
//! delegates work everywhere.
//!
//! ## `encode` versus `encode_by_ref`
//!
//! Only [`Encode::encode_by_ref`] is implemented; `encode` keeps its default,
//! deliberately. The consuming form's advantage is backend-specific and does
//! not survive a `DB`-generic impl: SQLite's `Encode for Vec<u8>` really does
//! move the vector into an `Arc` instead of cloning it, so routing through
//! `Vec<u8>` would let a uniquely-owned [`shared::Bytes`](crate::shared::Bytes)
//! reach the driver with no copy at all — but PostgreSQL and MySQL both write
//! straight into the argument buffer from a `&[u8]`, so the same routing would
//! *add* an allocation and a copy on two backends out of three. Encoding from
//! a borrow costs one copy everywhere and never more.
//!
//! Encoding from a borrow is possible at all only because sqlx 0.9 dropped
//! the lifetime parameter from `Database::ArgumentBuffer`; a higher-ranked
//! `for<'a> &'a [u8]: Encode<'a, DB>` bound then lets `encode_by_ref` pass a
//! slice borrowed from `&self`. `sqlx-core`'s own `bstr` integration cannot do
//! this and calls `to_vec()` first, paying an extra allocation and copy per
//! encode.

use sqlx::{Database, Decode, Encode, Type, encode::IsNull, error::BoxDynError};

use crate::{
  Buffer, Utf8Buffer,
  compact::{Bytes as CompactBytes, Utf8Bytes as CompactUtf8Bytes},
  shared::{Bytes as SharedBytes, Utf8Bytes as SharedUtf8Bytes},
};

/// Fits a decoded row into a fixed-capacity [`Buffer`].
///
/// A row longer than [`INLINE_CAP`](crate::INLINE_CAP) is an error naming the
/// limit, never a truncation.
fn fit_buffer(bytes: &[u8]) -> Result<Buffer, BoxDynError> {
  Buffer::try_from(bytes).map_err(Into::into)
}

/// Fits a decoded row into a fixed-capacity [`Utf8Buffer`].
///
/// The limit counts bytes, not characters. UTF-8 validity is already
/// established by decoding through `String`, so this cannot fail for that
/// reason.
fn fit_utf8_buffer(text: &str) -> Result<Utf8Buffer, BoxDynError> {
  Utf8Buffer::try_from_str(text).map_err(Into::into)
}

macro_rules! byte_type {
  ($ty:ty) => {
    impl<DB> Type<DB> for $ty
    where
      DB: Database,
      [u8]: Type<DB>,
    {
      fn type_info() -> DB::TypeInfo {
        <&[u8] as Type<DB>>::type_info()
      }

      fn compatible(ty: &DB::TypeInfo) -> bool {
        <&[u8] as Type<DB>>::compatible(ty)
      }
    }

    impl<DB> Encode<'_, DB> for $ty
    where
      DB: Database,
      for<'a> &'a [u8]: Encode<'a, DB>,
    {
      fn encode_by_ref(&self, buf: &mut DB::ArgumentBuffer) -> Result<IsNull, BoxDynError> {
        <&[u8] as Encode<'_, DB>>::encode(self.as_slice(), buf)
      }

      fn size_hint(&self) -> usize {
        self.len()
      }
    }
  };
}

macro_rules! text_type {
  ($ty:ty) => {
    impl<DB> Type<DB> for $ty
    where
      DB: Database,
      str: Type<DB>,
    {
      fn type_info() -> DB::TypeInfo {
        <&str as Type<DB>>::type_info()
      }

      fn compatible(ty: &DB::TypeInfo) -> bool {
        <&str as Type<DB>>::compatible(ty)
      }
    }

    impl<DB> Encode<'_, DB> for $ty
    where
      DB: Database,
      for<'a> &'a str: Encode<'a, DB>,
    {
      fn encode_by_ref(&self, buf: &mut DB::ArgumentBuffer) -> Result<IsNull, BoxDynError> {
        <&str as Encode<'_, DB>>::encode(self.as_str(), buf)
      }

      fn size_hint(&self) -> usize {
        self.len()
      }
    }
  };
}

byte_type!(SharedBytes);
byte_type!(CompactBytes);
byte_type!(Buffer);

text_type!(SharedUtf8Bytes);
text_type!(CompactUtf8Bytes);
text_type!(Utf8Buffer);

impl<'r, DB> Decode<'r, DB> for SharedBytes
where
  DB: Database,
  Vec<u8>: Decode<'r, DB>,
{
  fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
    <Vec<u8> as Decode<'r, DB>>::decode(value).map(Self::from)
  }
}

impl<'r, DB> Decode<'r, DB> for CompactBytes
where
  DB: Database,
  Vec<u8>: Decode<'r, DB>,
{
  fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
    <Vec<u8> as Decode<'r, DB>>::decode(value).map(Self::from)
  }
}

impl<'r, DB> Decode<'r, DB> for Buffer
where
  DB: Database,
  Vec<u8>: Decode<'r, DB>,
{
  fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
    fit_buffer(&<Vec<u8> as Decode<'r, DB>>::decode(value)?)
  }
}

impl<'r, DB> Decode<'r, DB> for SharedUtf8Bytes
where
  DB: Database,
  String: Decode<'r, DB>,
{
  fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
    <String as Decode<'r, DB>>::decode(value).map(Self::from)
  }
}

impl<'r, DB> Decode<'r, DB> for CompactUtf8Bytes
where
  DB: Database,
  String: Decode<'r, DB>,
{
  fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
    <String as Decode<'r, DB>>::decode(value).map(Self::from)
  }
}

impl<'r, DB> Decode<'r, DB> for Utf8Buffer
where
  DB: Database,
  String: Decode<'r, DB>,
{
  fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
    fit_utf8_buffer(&<String as Decode<'r, DB>>::decode(value)?)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::INLINE_CAP;

  /// A row one byte over the inline capacity must be refused with the limit
  /// named, not silently shortened. This is the one place the integration
  /// could corrupt data.
  #[test]
  fn buffer_decode_refuses_one_byte_over_capacity() {
    let accepted = fit_buffer(&std::vec![b'x'; INLINE_CAP]).unwrap();
    assert_eq!(accepted.len(), INLINE_CAP);

    let rejected = fit_buffer(&std::vec![b'x'; INLINE_CAP + 1]).unwrap_err();
    let text = rejected.to_string();

    assert!(text.contains("63"), "{text}");
    assert!(text.contains(&INLINE_CAP.to_string()), "{text}");
  }

  #[test]
  fn utf8_buffer_decode_refuses_one_byte_over_capacity() {
    let accepted = fit_utf8_buffer(&"x".repeat(INLINE_CAP)).unwrap();
    assert_eq!(accepted.len(), INLINE_CAP);

    let rejected = fit_utf8_buffer(&"x".repeat(INLINE_CAP + 1)).unwrap_err();
    let text = rejected.to_string();

    assert!(text.contains("63"), "{text}");
    assert!(text.contains(&INLINE_CAP.to_string()), "{text}");
  }

  /// The ceiling is on bytes. A row that fits by character count but not by
  /// byte count must still be refused rather than split mid-character.
  #[test]
  fn utf8_buffer_decode_counts_bytes_not_characters() {
    let fits = "é".repeat(INLINE_CAP / 2);
    assert_eq!(fits.len(), INLINE_CAP);
    assert_eq!(fit_utf8_buffer(&fits).unwrap().as_str(), fits);

    let overflows = "é".repeat(INLINE_CAP / 2 + 1);
    assert!(overflows.chars().count() < INLINE_CAP);
    assert!(fit_utf8_buffer(&overflows).is_err());
  }

  /// Decoding a heap-sized row must move the delegate's allocation into the
  /// reference-counted representation rather than copying it again.
  #[test]
  fn heap_sized_decode_moves_the_delegates_allocation() {
    let row = std::vec![7u8; INLINE_CAP + 1];
    let address = row.as_ptr();
    let decoded = SharedBytes::from(row);

    assert!(decoded.is_heap());
    assert_eq!(decoded.as_slice().as_ptr(), address);

    let row = "é".repeat(INLINE_CAP);
    let address = row.as_ptr();
    let decoded = SharedUtf8Bytes::from(row);

    assert!(decoded.is_heap());
    assert_eq!(decoded.as_str().as_ptr(), address);
  }
}
