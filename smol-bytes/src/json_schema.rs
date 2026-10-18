use crate::{
  Buffer, BytesMut, Utf8Buffer, Utf8BytesMut,
  bytes::{RawBytes, strategy::ImmutableStorage},
  utf8_bytes::Utf8Bytes,
};
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use std::borrow::{Cow, ToOwned};

fn byte_schema(generator: &mut SchemaGenerator) -> Schema {
  let items = generator.subschema_for::<u8>();
  if generator.contract().is_deserialize() {
    json_schema!({
      "type": ["array", "string"],
      "items": items,
    })
  } else {
    json_schema!({
      "type": "array",
      "items": items,
    })
  }
}

impl JsonSchema for Buffer {
  fn schema_name() -> Cow<'static, str> {
    "Buffer".into()
  }

  fn schema_id() -> Cow<'static, str> {
    "smol_bytes::Buffer".into()
  }

  fn json_schema(generator: &mut SchemaGenerator) -> Schema {
    let mut schema = byte_schema(generator);
    schema.insert("maxItems".to_owned(), (crate::INLINE_CAP as u64).into());
    schema
  }
}

impl JsonSchema for BytesMut {
  fn schema_name() -> Cow<'static, str> {
    "BytesMut".into()
  }

  fn schema_id() -> Cow<'static, str> {
    "smol_bytes::BytesMut".into()
  }

  fn json_schema(generator: &mut SchemaGenerator) -> Schema {
    byte_schema(generator)
  }
}

impl<S> JsonSchema for RawBytes<S>
where
  RawBytes<S>: ImmutableStorage,
{
  fn schema_name() -> Cow<'static, str> {
    "Bytes".into()
  }

  fn schema_id() -> Cow<'static, str> {
    "smol_bytes::Bytes".into()
  }

  fn json_schema(generator: &mut SchemaGenerator) -> Schema {
    byte_schema(generator)
  }
}

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

  use schemars::generate::{Contract, SchemaSettings};
  use schemars::{JsonSchema, Schema, schema_for};
  use serde_json::json;

  use crate::{Buffer, Bytes, BytesMut, INLINE_CAP, Utf8Buffer, Utf8Bytes, Utf8BytesMut, compact};

  fn schema_for_contract<T: JsonSchema>(contract: Contract) -> Schema {
    let mut settings = SchemaSettings::draft2020_12();
    settings.contract = contract;
    settings.into_generator().into_root_schema_for::<T>()
  }

  #[test]
  fn byte_wrappers_describe_serialized_arrays_and_deserialized_strings() {
    for schema in [
      schema_for_contract::<Buffer>(Contract::Serialize),
      schema_for_contract::<BytesMut>(Contract::Serialize),
      schema_for_contract::<Bytes>(Contract::Serialize),
      schema_for_contract::<compact::Bytes>(Contract::Serialize),
    ] {
      assert_eq!(schema.get("type"), Some(&json!("array")));
      assert!(schema.get("items").is_some());
    }

    for schema in [
      schema_for_contract::<Buffer>(Contract::Deserialize),
      schema_for_contract::<BytesMut>(Contract::Deserialize),
      schema_for_contract::<Bytes>(Contract::Deserialize),
      schema_for_contract::<compact::Bytes>(Contract::Deserialize),
    ] {
      assert_eq!(schema.get("type"), Some(&json!(["array", "string"])));
    }

    let buffer_schema = schema_for_contract::<Buffer>(Contract::Serialize);
    assert_eq!(buffer_schema.get("maxItems"), Some(&json!(INLINE_CAP)));
  }

  #[test]
  fn shared_and_compact_bytes_have_one_schema_identity() {
    assert_eq!(
      <Bytes as JsonSchema>::schema_name(),
      <compact::Bytes as JsonSchema>::schema_name()
    );
    assert_eq!(
      <Bytes as JsonSchema>::schema_id(),
      <compact::Bytes as JsonSchema>::schema_id()
    );
    assert_eq!(schema_for!(Bytes), schema_for!(compact::Bytes));
  }

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
