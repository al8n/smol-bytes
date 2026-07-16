use js_sys::JsString;
use wasm_bindgen::JsError;

const PUSH_ERROR: &str = "push expects exactly one Unicode scalar value";

/// Converts exactly one well-formed JavaScript UTF-16 scalar to Rust `char`.
///
/// This intentionally examines code units instead of converting through Rust
/// `String`, because that conversion replaces isolated surrogates.
pub(crate) fn js_string_to_char(value: &JsString) -> Result<char, JsError> {
  let code_unit = |index| value.char_code_at(index) as u16;
  match value.length() {
    1 => {
      let first = code_unit(0);
      if (0xD800..=0xDFFF).contains(&first) {
        return Err(JsError::new(PUSH_ERROR));
      }
      char::from_u32(first as u32).ok_or_else(|| JsError::new(PUSH_ERROR))
    }
    2 => {
      let high = code_unit(0);
      let low = code_unit(1);
      if !(0xD800..=0xDBFF).contains(&high) || !(0xDC00..=0xDFFF).contains(&low) {
        return Err(JsError::new(PUSH_ERROR));
      }
      let scalar = 0x1_0000 + (((high as u32 - 0xD800) << 10) | (low as u32 - 0xDC00));
      char::from_u32(scalar).ok_or_else(|| JsError::new(PUSH_ERROR))
    }
    _ => Err(JsError::new(PUSH_ERROR)),
  }
}
