use smol_bytes::{Utf8Buffer, Utf8EditError, Utf8IndexError};

#[test]
fn fixed_utf8_edits_and_drains_work_without_allocation() {
  let mut text = Utf8Buffer::try_from("aé🦀z").unwrap();
  text.try_insert_str(1, "€").unwrap();
  text.try_replace_range(1..4, "é").unwrap();
  text.retain(|character| character != 'é');
  assert_eq!(text.as_str(), "a🦀z");

  {
    let mut drain = text.try_drain(1..5).unwrap();
    assert_eq!(drain.next(), Some('🦀'));
  }
  assert_eq!(text.as_str(), "az");

  assert_eq!(
    text.try_insert_str(9, "x"),
    Err(Utf8EditError::Index(Utf8IndexError::IndexOutOfBounds(
      smol_bytes::OutOfBounds::new(9, 2)
    )))
  );
}
