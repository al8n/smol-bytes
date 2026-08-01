//! [`async-graphql`](https://docs.rs/async-graphql) scalar bindings.
//!
//! ## Which shape each type gets
//!
//! Byte types map to `Value::Binary`, UTF-8 types map to `Value::String`.
//! The split is what lets [`Utf8Bytes`](crate::shared::Utf8Bytes) sit in a text
//! position in a schema instead of being an opaque blob: a `String`-shaped
//! scalar is readable in a query document and survives a JSON transport, which
//! `Value::Binary` does not. Nothing in the GraphQL grammar produces a binary
//! literal and `serde_json` never calls `visit_bytes`, so `Value::Binary` only
//! ever arrives through a binary variables encoding such as CBOR.
//!
//! ## Scalar names
//!
//! Every scalar registered here is named
//!
//! > `Smol` + *strategy, when the crate spells the type once per strategy* +
//! > *the Rust type name*
//!
//! giving `SmolSharedBytes`, `SmolCompactBytes`, `SmolSharedUtf8Bytes` and
//! `SmolCompactUtf8Bytes`.
//!
//! The `Smol` prefix is not decoration. `async-graphql` registers its own
//! `bytes::Bytes` under the GraphQL name `Bytes`, unconditionally — that
//! integration is not feature-gated, because `Value::Binary` holds a
//! `bytes::Bytes`. A schema using both it and a scalar of ours named `Bytes`
//! would declare two different types under one name. The same hazard already
//! exists inside `async-graphql`, where `Duration` is claimed by
//! `chrono::Duration` and by `jiff::Span`. Prefixing sidesteps it: the only
//! name `async-graphql` 7.x registers that starts with `Smol` is `SmolStr`,
//! and no type here is called `Str`.
//!
//! The strategy segment is required because `Bytes` and `Utf8Bytes` each exist
//! twice, once per [strategy](crate::compact), and the two differ in
//! allocation behaviour — they are genuinely different scalars, not aliases.
//!
//! ## The capped types have no scalar
//!
//! [`Buffer`](crate::Buffer) and [`Utf8Buffer`](crate::Utf8Buffer) hold their
//! contents inline and are capped at [`INLINE_CAP`](crate::INLINE_CAP) bytes;
//! their `TryFrom` conversions refuse anything longer. A GraphQL input carries
//! data of unbounded length that the program does not choose, so decoding one
//! into a capped type turns an ordinary long value into a runtime failure on
//! data nobody picked. The four types above accept any length. A caller that
//! knows its payload is short enough can convert after parsing, where the
//! length is its own to check.
//!
//! ## Copying
//!
//! Each `impl` documents its own behaviour. In summary, on the byte side
//! [`shared::Bytes`](crate::shared::Bytes) is allocation-free in *both*
//! directions, because `Value::Binary` carries the very `bytes::Bytes` that
//! the shared strategy stores. On the text side no direction can be
//! allocation-free, because `Value::String` owns a `String` and `to_value`
//! only has `&self` to hand it.

use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

use crate::{
  compact::{Bytes as CompactBytes, Utf8Bytes as CompactUtf8Bytes},
  shared::{Bytes as SharedBytes, Utf8Bytes as SharedUtf8Bytes},
};

/// Binary data held in a `smol_bytes::shared::Bytes`, which stores up to 62
/// bytes inline and reference-counts anything larger. Both directions are
/// allocation-free for a heap-backed value: parsing moves the incoming
/// `bytes::Bytes` in without inspecting its length, and serialising hands back
/// a reference-counted clone of the same allocation. An inline value costs one
/// allocation of at most 62 bytes on the way out, which `Value::Binary` makes
/// unavoidable.
#[Scalar(name = "SmolSharedBytes")]
impl ScalarType for SharedBytes {
  fn parse(value: Value) -> InputValueResult<Self> {
    match value {
      Value::Binary(data) => Ok(Self::from(data)),
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    matches!(value, Value::Binary(_))
  }

  fn to_value(&self) -> Value {
    Value::Binary(self.clone().into_bytes())
  }
}

/// Binary data held in a `smol_bytes::compact::Bytes`, which inlines whatever
/// fits in 62 bytes rather than keeping a heap allocation alive. Parsing a
/// longer payload moves the incoming `bytes::Bytes` in unchanged; a shorter one
/// is copied inline and the incoming allocation released, which is the whole
/// point of the compact strategy. Serialising a heap-backed value is
/// allocation-free; an inline one costs one allocation of at most 62 bytes.
#[Scalar(name = "SmolCompactBytes")]
impl ScalarType for CompactBytes {
  fn parse(value: Value) -> InputValueResult<Self> {
    match value {
      Value::Binary(data) => Ok(Self::from(data)),
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    matches!(value, Value::Binary(_))
  }

  fn to_value(&self) -> Value {
    Value::Binary(self.clone().into_bytes())
  }
}

/// UTF-8 text held in a `smol_bytes::shared::Utf8Bytes`. Parsing takes the
/// `String` by value: above 62 bytes its allocation is moved into the
/// reference-counted representation, at or below 62 bytes it is copied inline
/// and released. The UTF-8 invariant costs nothing here — the source is already
/// a `String`, and the type reads itself back with `from_utf8_unchecked`.
/// Serialising always allocates, because `Value::String` owns its `String`.
///
/// Only `Value::String` is accepted. `Value::Binary` carries no UTF-8
/// guarantee, and validating it inside `is_valid` — which `async-graphql`
/// documents as a cheap pre-check run during query validation — would scan
/// every byte of the payload, then scan it again in `parse`. Decode such input
/// through `SmolSharedBytes` and this crate's checked `TryFrom` conversions
/// instead.
#[Scalar(name = "SmolSharedUtf8Bytes")]
impl ScalarType for SharedUtf8Bytes {
  fn parse(value: Value) -> InputValueResult<Self> {
    match value {
      Value::String(s) => Ok(Self::from(s)),
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    matches!(value, Value::String(_))
  }

  fn to_value(&self) -> Value {
    Value::String(self.as_str().to_owned())
  }
}

/// UTF-8 text held in a `smol_bytes::compact::Utf8Bytes`. Parsing behaves
/// exactly as `SmolSharedUtf8Bytes` — for a value built from a `String` the
/// inline/heap split is decided by length, not by strategy — but the compact
/// strategy keeps collapsing the value back inline as it shrinks.
/// `Value::Binary` is rejected for the same reason as `SmolSharedUtf8Bytes`.
#[Scalar(name = "SmolCompactUtf8Bytes")]
impl ScalarType for CompactUtf8Bytes {
  fn parse(value: Value) -> InputValueResult<Self> {
    match value {
      Value::String(s) => Ok(Self::from(s)),
      _ => Err(InputValueError::expected_type(value)),
    }
  }

  fn is_valid(value: &Value) -> bool {
    matches!(value, Value::String(_))
  }

  fn to_value(&self) -> Value {
    Value::String(self.as_str().to_owned())
  }
}

#[cfg(test)]
mod tests {
  use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};

  use super::*;
  use crate::INLINE_CAP;

  fn binary(len: usize) -> Value {
    Value::Binary(::bytes::Bytes::from(std::vec![b'x'; len]))
  }

  /// The shared strategy promises a zero-copy handover with `bytes::Bytes`.
  /// `parse` must therefore keep the incoming allocation rather than rebuild
  /// one, whatever the length — a length check here would be the compact
  /// strategy's behaviour, not this one's.
  #[test]
  fn shared_bytes_parse_moves_the_allocation() {
    for len in [1, INLINE_CAP, INLINE_CAP + 1, 4096] {
      let source = ::bytes::Bytes::from(std::vec![7u8; len]);
      let address = source.as_ptr();

      let parsed = <SharedBytes as ScalarType>::parse(Value::Binary(source)).unwrap();

      assert!(parsed.is_heap(), "len {len} left the shared strategy");
      assert_eq!(parsed.as_slice().as_ptr(), address, "len {len} was copied");
    }
  }

  /// `to_value` only has `&self`, but a heap-backed shared value can still
  /// hand `Value::Binary` a reference-counted clone of its own allocation.
  #[test]
  fn shared_bytes_to_value_shares_the_allocation() {
    let value = SharedBytes::from(std::vec![7u8; INLINE_CAP + 1]);
    let address = value.as_slice().as_ptr();

    match <SharedBytes as ScalarType>::to_value(&value) {
      Value::Binary(data) => assert_eq!(data.as_ptr(), address),
      other => panic!("expected Value::Binary, got {other:?}"),
    }
  }

  /// The compact strategy's whole purpose is to release a heap allocation it
  /// no longer needs, so unlike the shared one it must inline on the way in.
  #[test]
  fn compact_bytes_parse_inlines_up_to_the_limit() {
    let inlined = <CompactBytes as ScalarType>::parse(binary(INLINE_CAP)).unwrap();
    assert!(inlined.is_inline());

    let source = ::bytes::Bytes::from(std::vec![7u8; INLINE_CAP + 1]);
    let address = source.as_ptr();
    let kept = <CompactBytes as ScalarType>::parse(Value::Binary(source)).unwrap();

    assert!(kept.is_heap());
    assert_eq!(kept.as_slice().as_ptr(), address);
  }

  #[test]
  fn text_scalars_refuse_binary_and_byte_scalars_refuse_text() {
    let bytes = binary(4);
    let text = Value::String("data".into());

    assert!(!<SharedUtf8Bytes as ScalarType>::is_valid(&bytes));
    assert!(<SharedUtf8Bytes as ScalarType>::parse(bytes.clone()).is_err());
    assert!(!<CompactUtf8Bytes as ScalarType>::is_valid(&bytes));
    assert!(<CompactUtf8Bytes as ScalarType>::parse(bytes).is_err());

    assert!(!<SharedBytes as ScalarType>::is_valid(&text));
    assert!(<SharedBytes as ScalarType>::parse(text.clone()).is_err());
    assert!(!<CompactBytes as ScalarType>::is_valid(&text));
    assert!(<CompactBytes as ScalarType>::parse(text).is_err());
  }

  #[test]
  fn every_scalar_round_trips() {
    let short = b"smol".as_slice();
    let long = &std::vec![9u8; 4096][..];

    for payload in [short, long] {
      let shared = SharedBytes::copy_from_slice(payload);
      let value = <SharedBytes as ScalarType>::to_value(&shared);
      assert_eq!(
        <SharedBytes as ScalarType>::parse(value)
          .unwrap()
          .as_slice(),
        payload
      );

      let compact = CompactBytes::copy_from_slice(payload);
      let value = <CompactBytes as ScalarType>::to_value(&compact);
      assert_eq!(
        <CompactBytes as ScalarType>::parse(value)
          .unwrap()
          .as_slice(),
        payload
      );
    }

    for payload in ["smol", &"é".repeat(4096)] {
      let shared = SharedUtf8Bytes::from(payload);
      let value = <SharedUtf8Bytes as ScalarType>::to_value(&shared);
      assert_eq!(
        <SharedUtf8Bytes as ScalarType>::parse(value)
          .unwrap()
          .as_str(),
        payload
      );

      let compact = CompactUtf8Bytes::from(payload);
      let value = <CompactUtf8Bytes as ScalarType>::to_value(&compact);
      assert_eq!(
        <CompactUtf8Bytes as ScalarType>::parse(value)
          .unwrap()
          .as_str(),
        payload
      );
    }
  }

  struct Query;

  #[Object]
  impl Query {
    async fn shared_bytes(&self) -> SharedBytes {
      SharedBytes::new()
    }

    async fn compact_bytes(&self) -> CompactBytes {
      CompactBytes::new()
    }

    async fn shared_utf8_bytes(&self) -> SharedUtf8Bytes {
      SharedUtf8Bytes::new()
    }

    async fn compact_utf8_bytes(&self) -> CompactUtf8Bytes {
      CompactUtf8Bytes::new()
    }

    /// Present so the schema also carries the `Bytes` scalar that
    /// `async-graphql` registers for `bytes::Bytes`, which is the name our
    /// prefix exists to stay clear of.
    async fn upstream_bytes(&self) -> ::bytes::Bytes {
      ::bytes::Bytes::new()
    }
  }

  /// Pins the four registered names, and pins them in a schema that also
  /// contains `async-graphql`'s own `Bytes`. A rename is a breaking schema
  /// change, and an unprefixed name would put two different types under one
  /// name here rather than the five distinct scalars this asserts.
  #[test]
  fn sdl_registers_four_prefixed_scalars_alongside_upstream_bytes() {
    let sdl = Schema::new(Query, EmptyMutation, EmptySubscription).sdl();

    let names = [
      "Bytes",
      "SmolSharedBytes",
      "SmolCompactBytes",
      "SmolSharedUtf8Bytes",
      "SmolCompactUtf8Bytes",
    ];
    for name in names {
      let declaration = std::format!("scalar {name}\n");
      assert_eq!(sdl.matches(&declaration).count(), 1, "{name} in {sdl}");
    }
  }
}
