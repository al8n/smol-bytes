use core::ops::{Bound, RangeBounds};
use std::{
  cell::Cell,
  error::Error,
  mem,
  panic::{self, AssertUnwindSafe},
};

use smol_bytes::{
  INLINE_CAP, OutOfBounds, RangeOutOfBounds, TryPutError, Utf8Buffer, Utf8BytesMut, Utf8Drain,
  Utf8EditError, Utf8IndexError,
};

struct CountingRange<'a> {
  start: usize,
  end: usize,
  start_calls: &'a Cell<usize>,
  end_calls: &'a Cell<usize>,
}

impl RangeBounds<usize> for CountingRange<'_> {
  fn start_bound(&self) -> Bound<&usize> {
    let calls = self.start_calls.get();
    assert_eq!(calls, 0, "start_bound called more than once");
    self.start_calls.set(calls + 1);
    Bound::Included(&self.start)
  }

  fn end_bound(&self) -> Bound<&usize> {
    let calls = self.end_calls.get();
    assert_eq!(calls, 0, "end_bound called more than once");
    self.end_calls.set(calls + 1);
    Bound::Excluded(&self.end)
  }
}

#[test]
fn insert_and_replace_support_mixed_unicode_and_all_length_relations() {
  macro_rules! exercise_edits {
    ($value:expr) => {{
      let mut value = $value;
      value.insert_str(1, "€");
      assert_eq!(value.as_str(), "a€é🦀z");

      value.replace_range(1..4, "é");
      assert_eq!(value.as_str(), "aéé🦀z");

      value.replace_range(1..3, "ê");
      assert_eq!(value.as_str(), "aêé🦀z");

      value.replace_range(.., "");
      assert!(value.is_empty());
      value.insert_str(0, "full");
      assert_eq!(value.as_str(), "full");
    }};
  }

  exercise_edits!(Utf8Buffer::try_from("aé🦀z").unwrap());
  exercise_edits!(Utf8BytesMut::from("aé🦀z"));
}

#[test]
fn fixed_edits_report_exact_atomic_errors_and_require_explicit_reclaim() {
  let mut text = Utf8Buffer::try_from("aéz").unwrap();
  let original = text.as_str().to_owned();
  assert_eq!(
    text.try_insert_str(2, "x"),
    Err(Utf8EditError::Index(Utf8IndexError::InvalidCharBoundary {
      index: 2,
    }))
  );
  assert_eq!(
    text.try_insert_str(99, "x"),
    Err(Utf8EditError::Index(Utf8IndexError::IndexOutOfBounds(
      OutOfBounds::new(99, original.len())
    )))
  );
  assert_eq!(
    text.try_replace_range(
      {
        let (start, end) = (3, 1);
        start..end
      },
      "x"
    ),
    Err(Utf8EditError::Index(Utf8IndexError::RangeOutOfBounds(
      RangeOutOfBounds::new(3, 1, original.len())
    )))
  );
  assert_eq!(
    text.try_replace_range(..=usize::MAX, "x"),
    Err(Utf8EditError::Index(Utf8IndexError::RangeOutOfBounds(
      RangeOutOfBounds::new(0, usize::MAX, original.len())
    )))
  );
  assert_eq!(text.as_str(), original);

  let full_text = "a".repeat(INLINE_CAP);
  let mut full = Utf8Buffer::try_from(full_text.as_str()).unwrap();
  assert_eq!(
    full.try_insert_str(0, "é"),
    Err(Utf8EditError::Capacity(TryPutError {
      requested: 2,
      available: 0,
    }))
  );
  assert_eq!(full.as_str(), full_text);

  let mut advanced = Utf8Buffer::try_from("abcdefghijkl").unwrap();
  let _head = advanced.split_to(4);
  let growth = "x".repeat(51);
  let capacity = advanced.capacity();
  assert_eq!(
    advanced.try_insert_str(0, growth.as_str()),
    Err(Utf8EditError::Capacity(TryPutError {
      requested: 51,
      available: 50,
    }))
  );
  assert_eq!(advanced.as_str(), "efghijkl");
  assert_eq!(advanced.capacity(), capacity);
  assert!(advanced.try_reclaim(growth.len()));
  advanced.try_insert_str(0, growth.as_str()).unwrap();
  assert_eq!(advanced.as_str(), format!("{growth}efghijkl"));
}

#[test]
fn growable_edits_are_index_fallible_and_can_promote_to_heap() {
  let mut text = Utf8BytesMut::from("a".repeat(INLINE_CAP - 2));
  assert!(text.is_inline());
  text.try_insert_str(text.len(), "🦀").unwrap();
  assert!(text.is_heap());
  assert_eq!(text.len(), INLINE_CAP + 2);

  let before = text.as_str().to_owned();
  assert_eq!(
    text.try_insert_str(INLINE_CAP - 1, "x"),
    Err(Utf8IndexError::InvalidCharBoundary {
      index: INLINE_CAP - 1,
    })
  );
  assert_eq!(text.as_str(), before);

  text.replace_range(0..4, "é");
  assert_eq!(text.as_str(), format!("é{}🦀", "a".repeat(INLINE_CAP - 6)));
}

#[test]
fn retain_is_linear_panic_safe_and_preserves_views() {
  let mut text = Utf8Buffer::try_from("aé🦀b").unwrap();
  let mut seen = Vec::new();
  text.retain(|character| {
    seen.push(character);
    character != 'é'
  });
  assert_eq!(seen, vec!['a', 'é', '🦀', 'b']);
  assert_eq!(text.as_str(), "a🦀b");

  let mut advanced = Utf8Buffer::try_from("abcdef").unwrap();
  let _head = advanced.split_to(2);
  let pointer = advanced.as_str().as_ptr();
  let capacity = advanced.capacity();
  advanced.retain(|_| true);
  assert_eq!(advanced.as_str(), "cdef");
  assert_eq!(advanced.as_str().as_ptr(), pointer);
  assert_eq!(advanced.capacity(), capacity);

  let mut none = Utf8Buffer::try_from("aé🦀b").unwrap();
  none.retain(|_| false);
  assert!(none.is_empty());

  let mut heap = Utf8BytesMut::from("aé".repeat(40));
  assert!(heap.is_heap());
  heap.retain(|character| character != 'é');
  assert!(heap.is_heap());
  assert_eq!(heap.as_str(), "a".repeat(40));

  let mut panic_before = Utf8Buffer::try_from("aé🦀z").unwrap();
  let result = panic::catch_unwind(AssertUnwindSafe(|| {
    panic_before.retain(|_| panic!("before any decision"));
  }));
  assert!(result.is_err());
  assert_eq!(panic_before.as_str(), "aé🦀z");

  let mut panic_after_rejection = Utf8Buffer::try_from("aé🦀z").unwrap();
  let result = panic::catch_unwind(AssertUnwindSafe(|| {
    panic_after_rejection.retain(|character| match character {
      'a' => true,
      'é' => false,
      '🦀' => panic!("after a rejection"),
      _ => true,
    });
  }));
  assert!(result.is_err());
  assert_eq!(panic_after_rejection.as_str(), "a🦀z");
}

#[test]
fn retain_panic_after_multibyte_compaction_preserves_current_and_suffix() {
  let mut fixed = Utf8Buffer::try_from("prefix-é🦀€x").unwrap();
  let _head = fixed.split_to("prefix-".len());
  let fixed_pointer = fixed.as_str().as_ptr();
  let fixed_capacity = fixed.capacity();
  let result = panic::catch_unwind(AssertUnwindSafe(|| {
    fixed.retain(|character| match character {
      'é' => false,
      '🦀' | '€' => true,
      'x' => panic!("after multibyte compaction"),
      _ => unreachable!(),
    });
  }));
  assert!(result.is_err());
  assert_eq!(fixed.as_str(), "🦀€x");
  assert_eq!(fixed.as_str().as_ptr(), fixed_pointer);
  assert_eq!(fixed.capacity(), fixed_capacity);

  let mut growable = Utf8BytesMut::with_capacity(128);
  growable.push_str("prefix-é🦀€x");
  let _head = growable.split_to("prefix-".len());
  let growable_pointer = growable.as_str().as_ptr();
  let growable_capacity = growable.capacity();
  let result = panic::catch_unwind(AssertUnwindSafe(|| {
    growable.retain(|character| match character {
      'é' => false,
      '🦀' | '€' => true,
      'x' => panic!("after multibyte compaction"),
      _ => unreachable!(),
    });
  }));
  assert!(result.is_err());
  assert!(growable.is_heap());
  assert_eq!(growable.as_str(), "🦀€x");
  assert_eq!(growable.as_str().as_ptr(), growable_pointer);
  assert_eq!(growable.capacity(), growable_capacity);
}

#[test]
fn drain_iterates_bidirectionally_and_removes_its_original_range() {
  let mut text = Utf8Buffer::try_from("aé🦀z").unwrap();
  {
    let mut drain: Utf8Drain<'_> = text.drain(1..7);
    assert_eq!(drain.as_str(), "é🦀");
    assert_eq!(AsRef::<str>::as_ref(&drain), "é🦀");
    assert_eq!(AsRef::<[u8]>::as_ref(&drain), "é🦀".as_bytes());
    assert_eq!(format!("{drain:?}"), "Utf8Drain(\"é🦀\")");
    assert_eq!(drain.next(), Some('é'));
    assert_eq!(drain.next_back(), Some('🦀'));
    assert_eq!(drain.next(), None);
    assert_eq!(drain.next_back(), None);
  }
  assert_eq!(text.as_str(), "az");

  let mut empty = Utf8Buffer::try_from("abc").unwrap();
  {
    let drain = empty.drain(1..1);
    assert!(drain.as_str().is_empty());
  }
  assert_eq!(empty.as_str(), "abc");
  {
    let drain = empty.drain(..);
    assert_eq!(drain.collect::<String>(), "abc");
  }
  assert!(empty.is_empty());

  let mut partial = Utf8Buffer::try_from("abcdef").unwrap();
  {
    let mut drain = partial.drain(1..5);
    assert_eq!(drain.next(), Some('b'));
  }
  assert_eq!(partial.as_str(), "af");
}

#[test]
fn drain_handles_panics_forget_validation_and_advanced_storage() {
  let mut panicking = Utf8Buffer::try_from("abcd").unwrap();
  let result = panic::catch_unwind(AssertUnwindSafe(|| {
    let mut drain = panicking.drain(1..3);
    assert_eq!(drain.next(), Some('b'));
    panic!("consumer panic");
  }));
  assert!(result.is_err());
  assert_eq!(panicking.as_str(), "ad");

  let mut forgotten = Utf8Buffer::try_from("abcd").unwrap();
  let mut drain = forgotten.drain(1..3);
  assert_eq!(drain.next(), Some('b'));
  mem::forget(drain);
  assert_eq!(forgotten.as_str(), "abcd");

  let mut invalid = Utf8Buffer::try_from("aéz").unwrap();
  assert!(matches!(
    invalid.try_drain(2..3),
    Err(Utf8IndexError::InvalidCharBoundary { index: 2 })
  ));
  assert_eq!(invalid.as_str(), "aéz");

  let mut advanced = Utf8Buffer::try_from("abcdef").unwrap();
  let _head = advanced.split_to(2);
  let fixed_pointer = advanced.as_str().as_ptr();
  let fixed_capacity = advanced.capacity();
  {
    let mut drain = advanced.drain(1..3);
    assert_eq!(drain.next_back(), Some('e'));
  }
  assert_eq!(advanced.as_str(), "cf");
  assert_eq!(advanced.as_str().as_ptr(), fixed_pointer);
  assert_eq!(advanced.capacity(), fixed_capacity);

  let mut heap = Utf8BytesMut::with_capacity(128);
  heap.push_str("abcdef");
  let heap_pointer = heap.as_str().as_ptr();
  let heap_capacity = heap.capacity();
  {
    let mut drain = heap.drain(1..5);
    assert_eq!(drain.next(), Some('b'));
  }
  assert!(heap.is_heap());
  assert_eq!(heap.as_str(), "af");
  assert_eq!(heap.as_str().as_ptr(), heap_pointer);
  assert_eq!(heap.capacity(), heap_capacity);
}

#[test]
fn replace_and_drain_normalize_hostile_ranges_once_before_mutation() {
  let start_calls = Cell::new(0);
  let end_calls = Cell::new(0);
  let range = CountingRange {
    start: 1,
    end: 10,
    start_calls: &start_calls,
    end_calls: &end_calls,
  };
  let mut replacement = Utf8Buffer::try_from("aé€🦀z").unwrap();
  replacement.try_replace_range(range, "x").unwrap();
  assert_eq!(start_calls.get(), 1);
  assert_eq!(end_calls.get(), 1);
  assert_eq!(replacement.as_str(), "axz");

  let start_calls = Cell::new(0);
  let end_calls = Cell::new(0);
  let range = CountingRange {
    start: 1,
    end: 10,
    start_calls: &start_calls,
    end_calls: &end_calls,
  };
  let mut drained = Utf8Buffer::try_from("aé€🦀z").unwrap();
  {
    let drain = drained.try_drain(range).unwrap();
    assert_eq!(drain.as_str(), "é€🦀");
  }
  assert_eq!(start_calls.get(), 1);
  assert_eq!(end_calls.get(), 1);
  assert_eq!(drained.as_str(), "az");
}

#[test]
fn reverse_drain_handles_one_to_four_byte_scalars() {
  let mut text = Utf8Buffer::try_from("aé€🦀").unwrap();
  {
    let mut drain = text.drain(..);
    assert_eq!(drain.next_back(), Some('🦀'));
    assert_eq!(drain.next_back(), Some('€'));
    assert_eq!(drain.next_back(), Some('é'));
    assert_eq!(drain.next_back(), Some('a'));
    assert_eq!(drain.next_back(), None);
  }
  assert!(text.is_empty());
}

#[test]
fn utf8_edit_error_delegates_display_source_and_conversions() {
  let index_error: Utf8EditError = Utf8IndexError::InvalidCharBoundary { index: 2 }.into();
  assert_eq!(
    index_error.to_string(),
    "index 2 does not lie on a UTF-8 character boundary"
  );
  assert!(Error::source(&index_error).is_some());

  let capacity_error: Utf8EditError = TryPutError {
    requested: 3,
    available: 1,
  }
  .into();
  assert_eq!(
    capacity_error.to_string(),
    "Not enough bytes remaining in buffer to write value (requested 3 but only 1 available)"
  );
  assert!(Error::source(&capacity_error).is_some());
}
