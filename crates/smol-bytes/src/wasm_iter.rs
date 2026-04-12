use wasm_bindgen::prelude::*;

/// Iterator over bytes, compatible with the JS iterator protocol.
///
/// Each call to `next()` returns `{ value: number, done: false }` or `{ done: true }`.
/// Attach `Symbol.iterator` to use with `for...of` loops.
#[allow(missing_docs)]
#[wasm_bindgen(js_name = "ByteIterator")]
pub struct ByteIterator {
  data: Vec<u8>,
  index: usize,
}

impl ByteIterator {
  pub(crate) fn new(data: Vec<u8>) -> Self {
    Self { data, index: 0 }
  }
}

#[allow(missing_docs)]
#[wasm_bindgen(js_class = "ByteIterator")]
impl ByteIterator {
  /// Return the next `{ value, done }` object per the JS iterator protocol.
  ///
  /// Returns `{ value: number, done: false }` while bytes remain,
  /// then `{ done: true }` when exhausted.
  pub fn next(&mut self) -> JsValue {
    if self.index < self.data.len() {
      let val = self.data[self.index];
      self.index += 1;
      let obj = js_sys::Object::new();
      let _ = js_sys::Reflect::set(&obj, &"value".into(), &JsValue::from(val));
      let _ = js_sys::Reflect::set(&obj, &"done".into(), &JsValue::FALSE);
      obj.into()
    } else {
      let obj = js_sys::Object::new();
      let _ = js_sys::Reflect::set(&obj, &"done".into(), &JsValue::TRUE);
      obj.into()
    }
  }
}

/// Iterator over characters, compatible with the JS iterator protocol.
///
/// Each call to `next()` returns `{ value: string, done: false }` or `{ done: true }`.
/// Attach `Symbol.iterator` to use with `for...of` loops.
#[allow(missing_docs)]
#[wasm_bindgen(js_name = "CharIterator")]
pub struct CharIterator {
  chars: Vec<String>,
  index: usize,
}

impl CharIterator {
  pub(crate) fn new(s: &str) -> Self {
    Self {
      chars: s.chars().map(|c| c.to_string()).collect(),
      index: 0,
    }
  }
}

#[allow(missing_docs)]
#[wasm_bindgen(js_class = "CharIterator")]
impl CharIterator {
  /// Return the next `{ value, done }` object per the JS iterator protocol.
  ///
  /// Returns `{ value: string, done: false }` while characters remain,
  /// then `{ done: true }` when exhausted.
  pub fn next(&mut self) -> JsValue {
    if self.index < self.chars.len() {
      let val = &self.chars[self.index];
      self.index += 1;
      let obj = js_sys::Object::new();
      let _ = js_sys::Reflect::set(&obj, &"value".into(), &JsValue::from_str(val));
      let _ = js_sys::Reflect::set(&obj, &"done".into(), &JsValue::FALSE);
      obj.into()
    } else {
      let obj = js_sys::Object::new();
      let _ = js_sys::Reflect::set(&obj, &"done".into(), &JsValue::TRUE);
      obj.into()
    }
  }
}
