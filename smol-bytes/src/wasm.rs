use js_sys::JsString;
use wasm_bindgen::JsError;

const PUSH_ERROR: &str = "push expects exactly one Unicode scalar value";
const STRING_ERROR: &str = "strings must not contain isolated UTF-16 surrogates";

/// Converts a well-formed JavaScript UTF-16 string to Rust `String`.
///
/// JavaScript permits isolated surrogate code units while Rust strings do not.
/// Validate the code units before asking wasm-bindgen to convert them so it
/// cannot silently replace malformed input with U+FFFD.
pub(crate) fn js_string_to_string(value: &JsString) -> Result<String, JsError> {
  let code_unit = |index| value.char_code_at(index) as u16;
  let len = value.length();
  let mut index = 0;

  while index < len {
    let current = code_unit(index);
    if (0xD800..=0xDBFF).contains(&current) {
      if index + 1 >= len || !(0xDC00..=0xDFFF).contains(&code_unit(index + 1)) {
        return Err(JsError::new(STRING_ERROR));
      }
      index += 2;
    } else if (0xDC00..=0xDFFF).contains(&current) {
      return Err(JsError::new(STRING_ERROR));
    } else {
      index += 1;
    }
  }

  value.as_string().ok_or_else(|| JsError::new(STRING_ERROR))
}

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
