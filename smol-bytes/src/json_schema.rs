use crate::{
  Utf8Buffer, Utf8BytesMut,
  bytes::{RawBytes, strategy::ImmutableStorage},
  utf8_bytes::Utf8Bytes,
};
use schemars::{JsonSchema, Schema, SchemaGenerator};
use std::borrow::Cow;

impl JsonSchema for Utf8Buffer {
  fn inline_schema() -> bool {
    <str as JsonSchema>::inline_schema()
  }

  fn schema_name() -> Cow<'static, str> {
    <str as JsonSchema>::schema_name()
  }

  fn schema_id() -> Cow<'static, str> {
    <str as JsonSchema>::schema_id()
  }

  fn json_schema(generator: &mut SchemaGenerator) -> Schema {
    <str as JsonSchema>::json_schema(generator)
  }
}

impl JsonSchema for Utf8BytesMut {
  fn inline_schema() -> bool {
    <str as JsonSchema>::inline_schema()
  }

  fn schema_name() -> Cow<'static, str> {
    <str as JsonSchema>::schema_name()
  }

  fn schema_id() -> Cow<'static, str> {
    <str as JsonSchema>::schema_id()
  }

  fn json_schema(generator: &mut SchemaGenerator) -> Schema {
    <str as JsonSchema>::json_schema(generator)
  }
}

impl<S> JsonSchema for Utf8Bytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn inline_schema() -> bool {
    <str as JsonSchema>::inline_schema()
  }

  fn schema_name() -> Cow<'static, str> {
    <str as JsonSchema>::schema_name()
  }

  fn schema_id() -> Cow<'static, str> {
    <str as JsonSchema>::schema_id()
  }

  fn json_schema(generator: &mut SchemaGenerator) -> Schema {
    <str as JsonSchema>::json_schema(generator)
  }
}

#[cfg(test)]
mod tests {
  use std::{collections::BTreeMap, string::String, vec::Vec};

  use schemars::schema_for;

  use crate::{Utf8Buffer, Utf8Bytes, Utf8BytesMut, compact};

  #[test]
  fn utf8_wrappers_have_string_schemas() {
    let string_schema = schema_for!(String);
    assert_eq!(schema_for!(Utf8Buffer), string_schema);
    assert_eq!(schema_for!(Utf8Bytes), string_schema);
    assert_eq!(schema_for!(compact::Utf8Bytes), string_schema);
    assert_eq!(schema_for!(Utf8BytesMut), string_schema);
  }

  #[test]
  fn utf8_bytes_containers_have_string_schemas() {
    assert_string_container_schemas::<Utf8Buffer>();
    assert_string_container_schemas::<Utf8Bytes>();
    assert_string_container_schemas::<compact::Utf8Bytes>();
    assert_string_container_schemas::<Utf8BytesMut>();
  }

  fn assert_string_container_schemas<T>()
  where
    T: schemars::JsonSchema + Ord,
  {
    assert_eq!(schema_for!(Option<T>), schema_for!(Option<String>));
    assert_eq!(schema_for!(Vec<T>), schema_for!(Vec<String>));
    assert_eq!(
      schema_for!(BTreeMap<T, T>),
      schema_for!(BTreeMap<String, String>)
    );
  }
}
