//! [`async-graphql`](https://docs.rs/async-graphql) scalar bindings.
//!
//! ## Which types, and which scalar
//!
//! | Rust type | GraphQL scalar | GraphQL value |
//! | --- | --- | --- |
//! | [`shared::Bytes`](crate::shared::Bytes) | `Bytes` | `Value::Binary` |
//! | [`compact::Bytes`](crate::compact::Bytes) | `Bytes` | `Value::Binary` |
//! | [`shared::Utf8Bytes`](crate::shared::Utf8Bytes) | `String` | `Value::String` |
//! | [`compact::Utf8Bytes`](crate::compact::Utf8Bytes) | `String` | `Value::String` |
//!
//! A scalar name is a wire type, and there are two of those here: a binary one
//! and a UTF-8 text one. [`shared`](crate::shared) and
//! [`compact`](crate::compact) are two strategies for holding the same wire
//! value, and which one a program picks is not something a schema can act on,
//! so both register under one name. Nothing is lost by sharing it: `parse` and
//! `to_value` resolve on the Rust type, so the type written in a field
//! signature is still the one that gets built.
//!
//! The byte/text split is what lets [`Utf8Bytes`](crate::shared::Utf8Bytes) sit
//! in a text position in a schema instead of being an opaque blob: a
//! `String`-shaped scalar is readable in a query document and survives a JSON
//! transport, which `Value::Binary` does not. Nothing in the GraphQL grammar
//! produces a binary literal and `serde_json` never calls `visit_bytes`, so
//! `Value::Binary` only ever arrives through a binary variables encoding such
//! as CBOR.
//!
//! The text scalars accept **only** `Value::String`. Accepting `Value::Binary`
//! would require UTF-8 validation, and keeping `is_valid` in agreement with
//! `parse` would mean running that validation inside `is_valid` too — which
//! `async-graphql` documents as a cheap pre-check run during query validation.
//! That is a second scan of the whole payload on every request.
//! `async-graphql`'s own `String` and `SmolStr` scalars are likewise
//! `Value::String`-only. Callers holding bytes should use the binary scalar and
//! this crate's checked `TryFrom` conversions.
//!
//! ## `String` is a built-in, and is reused rather than redeclared
//!
//! The five built-in scalars are implicitly defined by the specification, so a
//! schema that emits `scalar String` is malformed. Two independent mechanisms
//! in `async-graphql` keep that line from ever appearing here.
//! `Registry::add_system_types` registers `Boolean`, `Int`, `Float`, `String`
//! and `ID` before the walk reaches any user type, and `Registry::create_type`
//! drops a later claimant of a name a scalar already holds — so the text
//! scalars always resolve to the built-in rather than replacing it. And
//! `Registry::export_sdl` skips the five by name regardless of what is
//! registered under them.
//!
//! `Bytes` is not privileged that way. `async-graphql` registers its own
//! `bytes::Bytes` under `Bytes` unconditionally — that integration is not
//! feature-gated, because `Value::Binary` holds a `bytes::Bytes` — so a schema
//! holding it and these types declares `Bytes` exactly once, from whichever
//! claimant the walk reaches first. The description that survives therefore has
//! to be true of all of them, which is why the ones below describe the wire
//! type and never the strategy, and why the two claimants here carry the same
//! description: which of the two wins is then unobservable.
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
//! [`shared::Bytes`](crate::shared::Bytes) is allocation-free in *both*
//! directions for a heap-backed value: `parse` moves the incoming
//! `bytes::Bytes` in without inspecting its length, and `to_value` hands
//! `Value::Binary` a reference-counted clone of the same allocation. An inline
//! value costs one allocation of at most [`INLINE_CAP`](crate::INLINE_CAP)
//! bytes on the way out, which `Value::Binary` makes unavoidable.
//!
//! [`compact::Bytes`](crate::compact::Bytes) differs only on the way in: a
//! payload that fits inline is copied and the incoming allocation released,
//! which is that strategy's whole purpose.
//!
//! Neither text type can be allocation-free in either direction, because
//! `Value::String` owns a `String` and `to_value` only has `&self`. `parse`
//! does take the `String` by value, so above [`INLINE_CAP`](crate::INLINE_CAP)
//! bytes its allocation is moved in and at or below it is copied inline and
//! released — a length-driven split, identical for both strategies.

use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};

use crate::{
  compact::{Bytes as CompactBytes, Utf8Bytes as CompactUtf8Bytes},
  shared::{Bytes as SharedBytes, Utf8Bytes as SharedUtf8Bytes},
};

/// Binary data of any length.
#[Scalar(name = "Bytes")]
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

/// Binary data of any length.
#[Scalar(name = "Bytes")]
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

/// UTF-8 text of any length.
#[Scalar(name = "String")]
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

/// UTF-8 text of any length.
#[Scalar(name = "String")]
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
  use async_graphql::{EmptyMutation, EmptySubscription, InputType, Object, Schema};

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

  /// Sharing a scalar name is a schema-level collapse only. Every type still
  /// parses through its own `ScalarType` impl, including at lengths a capped
  /// type would have refused.
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

  /// The registry keeps one type per name and silently drops the rest, so a
  /// name claimed by four Rust types is registered from one of them. Argument
  /// decoding does not go through the registry: it calls `InputType::parse` on
  /// the type in the signature. Each of the four must therefore come back as
  /// itself, out of a `Value` any of the others could have produced.
  #[test]
  fn input_parse_resolves_on_the_rust_type() {
    let bytes = binary(4096);
    assert_eq!(
      <SharedBytes as InputType>::parse(Some(bytes.clone()))
        .unwrap()
        .len(),
      4096
    );
    assert_eq!(
      <CompactBytes as InputType>::parse(Some(bytes))
        .unwrap()
        .len(),
      4096
    );

    let text = Value::String("é".repeat(4096));
    assert_eq!(
      <SharedUtf8Bytes as InputType>::parse(Some(text.clone()))
        .unwrap()
        .as_str(),
      "é".repeat(4096)
    );
    assert_eq!(
      <CompactUtf8Bytes as InputType>::parse(Some(text))
        .unwrap()
        .as_str(),
      "é".repeat(4096)
    );
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

  struct Query;

  #[Object]
  impl Query {
    /// Every type appears in both an output and an input position, so the
    /// schema below is built from both `OutputType::create_type_info` and
    /// `InputType::create_type_info` for all four.
    async fn shared_bytes(&self, echo: SharedBytes) -> SharedBytes {
      echo
    }

    async fn compact_bytes(&self, echo: CompactBytes) -> CompactBytes {
      echo
    }

    async fn shared_utf8_bytes(&self, echo: SharedUtf8Bytes) -> SharedUtf8Bytes {
      echo
    }

    async fn compact_utf8_bytes(&self, echo: CompactUtf8Bytes) -> CompactUtf8Bytes {
      echo
    }

    /// Present so the schema also carries `async-graphql`'s own `Bytes`
    /// scalar for `bytes::Bytes`, making it a third claimant of that name.
    async fn upstream_bytes(&self) -> ::bytes::Bytes {
      ::bytes::Bytes::new()
    }
  }

  /// `Bytes` is claimed by three Rust types here and `String` by two plus the
  /// built-in, and a scalar may be declared once. `String` additionally may
  /// not be declared at all: it is one of the five scalars the specification
  /// defines implicitly, and redeclaring one is invalid SDL.
  #[test]
  fn sdl_declares_bytes_once_and_never_declares_string() {
    let sdl = Schema::new(Query, EmptyMutation, EmptySubscription).sdl();

    assert_eq!(sdl.matches("scalar Bytes\n").count(), 1, "{sdl}");
    assert_eq!(sdl.matches("scalar String").count(), 0, "{sdl}");
  }

  struct SharedOnly;

  #[Object(name = "Query")]
  impl SharedOnly {
    async fn value(&self) -> SharedBytes {
      SharedBytes::new()
    }
  }

  struct CompactOnly;

  #[Object(name = "Query")]
  impl CompactOnly {
    async fn value(&self) -> CompactBytes {
      CompactBytes::new()
    }
  }

  /// Only one claimant of `Bytes` survives registration, and which one is
  /// decided by the order the schema walk happens to take. That is tolerable
  /// only while the claimants are interchangeable, so these two schemas —
  /// identical but for the strategy behind the scalar — must emit the same
  /// SDL, description included. A description that named a storage strategy
  /// would be wrong for the other claimant and would break this.
  #[test]
  fn the_bytes_claimants_are_indistinguishable_in_a_schema() {
    let shared = Schema::new(SharedOnly, EmptyMutation, EmptySubscription).sdl();
    let compact = Schema::new(CompactOnly, EmptyMutation, EmptySubscription).sdl();

    assert_eq!(shared, compact);
  }
}
