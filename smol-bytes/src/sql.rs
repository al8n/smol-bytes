//! [`sqlx`](https://docs.rs/sqlx) `Type` / `Encode` / `Decode` bindings.
//!
//! ## Shape
//!
//! One impl trio per type, generic over `DB: Database`, delegating to the
//! standard-library type the driver already supports — `[u8]` and `&[u8]` for
//! the byte types, `str` and `&str` for the UTF-8 ones. Every backend that
//! supports those comes along for free; PostgreSQL, MySQL and SQLite all do.
//! This mirrors `sqlx-core`'s own `bstr` integration.
//!
//! The four types bound are [`shared::Bytes`](crate::shared::Bytes),
//! [`compact::Bytes`](crate::compact::Bytes),
//! [`shared::Utf8Bytes`](crate::shared::Utf8Bytes) and
//! [`compact::Utf8Bytes`](crate::compact::Utf8Bytes).
//!
//! ## PostgreSQL: the byte types need a prepared query
//!
//! Decoding borrows from the row, and `sqlx-postgres` refuses to lend `BYTEA`
//! out as a `&[u8]` in a simple (unprepared) query — there the value arrives
//! as `\x`-prefixed hex text, so the row holds no bytes to borrow. The error
//! reads *"unsupported decode to `&[u8]` of BYTEA in a simple query; use a
//! prepared query or decode to `Vec<u8>`"*.
//!
//! A statement is prepared whenever it carries an argument list, which
//! `query`, `query_as` and the `query!` macros always do, even with nothing
//! bound. The simple protocol is reached only by executing a statement that
//! has no argument list at all: `raw_sql`, or a SQL string handed straight to
//! an `Executor` method. Where one of those has to read bytes, decode the
//! column as a `Vec<u8>` and convert — at the cost of the allocation this
//! binding otherwise avoids:
//!
//! ```text
//! let raw: Vec<u8> = row.try_get("payload")?;
//! let payload = smol_bytes::shared::Bytes::from(raw);
//! ```
//!
//! Nothing else is restricted: PostgreSQL lends `&str` out under either
//! protocol, so the UTF-8 types are unaffected, and MySQL and SQLite lend both
//! unconditionally.
//!
//! ## SQLite: a decoded value is marked borrowed
//!
//! `SqliteValueRef::blob_borrowed` and `text_borrowed` record on the value
//! handle that its buffer was lent out, because SQLite may invalidate that
//! pointer when the same value is read as another type. The one consequence is
//! that `sqlite3_value_int64` and `sqlite3_value_double` on **that same
//! handle** then report `BorrowedBlobError`: decoding a value as bytes or text
//! and *then* as a number is what breaks. Decoding a row is unaffected — each
//! column arrives as its own handle and is decoded once.
//!
//! ## The capped types are not bound
//!
//! [`Buffer`](crate::Buffer) and [`Utf8Buffer`](crate::Utf8Buffer) hold their
//! contents inline and are capped at [`INLINE_CAP`](crate::INLINE_CAP) bytes;
//! their `TryFrom` conversions refuse anything longer. A database column
//! carries data of unbounded length that the program does not choose, so
//! decoding one into a capped type turns an ordinary long row into a runtime
//! failure on data nobody picked — and a `Decode` cannot even warn about it
//! ahead of time, because `Type::compatible` sees the SQL type and never the
//! value. The four types above accept any length. A caller that knows its
//! rows are short enough can convert after decoding, where the length bound is
//! its own to check and its own to report on.
//!
//! ## Why the delegate is a borrow
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
//! backend, and delegating is the only shape available rather than merely the
//! convenient one. What that leaves to choose is *which* delegate, and the
//! borrowed one is never worse. `&[u8]` and `&str` hand over a view of the
//! row, and this crate then performs the one unavoidable copy directly into
//! its own representation. `Vec<u8>` and `String` perform that copy into an
//! allocation of their own first: above 62 bytes the allocation is handed on
//! and the two routes come out even, but at or below 62 bytes it is allocated,
//! copied out of, and freed — for a value that was always going to live
//! inline. That is the allocation this crate exists to avoid, and it is the
//! whole of what the delegate decides.
//!
//! There is no third shape that tries the borrow and falls back to the owned
//! delegate. `decode` consumes the `ValueRef` by value, and neither
//! `sqlx_core::value::ValueRef` nor `Database::ValueRef` is bound by `Copy` or
//! `Clone`, so a generic impl has nothing left to make a second attempt with.
//!
//! ## Copying, per type and direction
//!
//! | Type | `decode`, at or below 62 bytes | `decode`, above 62 bytes | `encode_by_ref` |
//! | --- | --- | --- | --- |
//! | [`shared::Bytes`](crate::shared::Bytes) / [`compact::Bytes`](crate::compact::Bytes) | one copy row → inline, **no allocation** | one copy row → a fresh reference-counted allocation | one copy into the argument buffer |
//! | [`shared::Utf8Bytes`](crate::shared::Utf8Bytes) / [`compact::Utf8Bytes`](crate::compact::Utf8Bytes) | the same, after the driver has validated UTF-8 | the same, after the driver has validated UTF-8 | one copy into the argument buffer |
//!
//! The two strategies do not differ here: both store a value inline when it
//! fits at the moment of construction, and they part company only later, over
//! whether an operation that shrinks a heap-backed value converts it back.
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
  compact::{Bytes as CompactBytes, Utf8Bytes as CompactUtf8Bytes},
  shared::{Bytes as SharedBytes, Utf8Bytes as SharedUtf8Bytes},
};

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

    impl<'r, DB> Decode<'r, DB> for $ty
    where
      DB: Database,
      &'r [u8]: Decode<'r, DB>,
    {
      fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
        <&'r [u8] as Decode<'r, DB>>::decode(value).map(Self::from)
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

    impl<'r, DB> Decode<'r, DB> for $ty
    where
      DB: Database,
      &'r str: Decode<'r, DB>,
    {
      fn decode(value: DB::ValueRef<'r>) -> Result<Self, BoxDynError> {
        <&'r str as Decode<'r, DB>>::decode(value).map(Self::from)
      }
    }
  };
}

byte_type!(SharedBytes);
byte_type!(CompactBytes);

text_type!(SharedUtf8Bytes);
text_type!(CompactUtf8Bytes);

#[cfg(test)]
mod tests {
  use super::*;
  use crate::INLINE_CAP;

  /// The delegate is the borrow, pinned by the compiler: this caller offers the
  /// four impls nothing but `&'r [u8]` and `&'r str`, so it stops compiling the
  /// moment one of them asks for an owned `Vec<u8>` or `String` instead.
  fn _decode_delegates_are_borrows<'r, DB>()
  where
    DB: Database,
    &'r [u8]: Decode<'r, DB>,
    &'r str: Decode<'r, DB>,
  {
    fn decodes<'r, DB: Database, T: Decode<'r, DB>>() {}

    decodes::<DB, SharedBytes>();
    decodes::<DB, CompactBytes>();
    decodes::<DB, SharedUtf8Bytes>();
    decodes::<DB, CompactUtf8Bytes>();
  }

  /// A row that fits inline must arrive with no heap allocation behind it. The
  /// borrowed delegate cannot own one — `&[u8]` and `&str` are views of the
  /// row — so the single copy goes straight into the inline buffer, and
  /// `is_inline` on the result is the whole accounting.
  #[test]
  fn inline_sized_decode_allocates_nothing() {
    let row = std::vec![7u8; INLINE_CAP];

    let decoded = SharedBytes::from(row.as_slice());
    assert!(decoded.is_inline());
    assert_eq!(decoded.as_slice(), row.as_slice());

    let decoded = CompactBytes::from(row.as_slice());
    assert!(decoded.is_inline());
    assert_eq!(decoded.as_slice(), row.as_slice());

    // Thirty-one two-byte characters land exactly on the cap.
    let row = "é".repeat(INLINE_CAP / 2);
    assert_eq!(row.len(), INLINE_CAP);

    let decoded = SharedUtf8Bytes::from(row.as_str());
    assert!(decoded.is_inline());
    assert_eq!(decoded.as_str(), row);

    let decoded = CompactUtf8Bytes::from(row.as_str());
    assert!(decoded.is_inline());
    assert_eq!(decoded.as_str(), row);
  }

  /// The other half of the table: above the cap the borrow is copied into an
  /// allocation of this crate's own, so the value carries the row's contents
  /// without aliasing a buffer the driver is still free to reuse.
  #[test]
  fn heap_sized_decode_copies_into_its_own_allocation() {
    let row = std::vec![7u8; INLINE_CAP + 1];

    let decoded = SharedBytes::from(row.as_slice());
    assert!(decoded.is_heap());
    assert_ne!(decoded.as_slice().as_ptr(), row.as_ptr());
    assert_eq!(decoded.as_slice(), row.as_slice());

    let row = "é".repeat(INLINE_CAP);

    let decoded = SharedUtf8Bytes::from(row.as_str());
    assert!(decoded.is_heap());
    assert_ne!(decoded.as_str().as_ptr(), row.as_ptr());
    assert_eq!(decoded.as_str(), row);
  }
}
