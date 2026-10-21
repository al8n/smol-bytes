#![cfg(all(feature = "serde", feature = "std"))]
#![warn(rust_2018_idioms)]

//! Heap allocations made while deserializing `Utf8Bytes`.
//!
//! This binary installs a counting global allocator. Counting is per thread
//! and the binary holds a single test, so no allocation made elsewhere in the
//! process can be attributed to a measurement.

use std::{
  alloc::{GlobalAlloc, Layout, System},
  cell::Cell,
};

use serde_core::de::DeserializeOwned;
use smol_bytes::{INLINE_CAP, compact, shared};

thread_local! {
  // `Cell<usize>` has no destructor, so the key stays accessible for the whole
  // life of the thread, including from inside the allocator.
  static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn record_allocation() {
  ALLOCATIONS.with(|count| count.set(count.get() + 1));
}

struct CountingAllocator;

// SAFETY: every method forwards its arguments unchanged to `System`, so the
// `GlobalAlloc` contract that callers uphold is `System`'s own.
unsafe impl GlobalAlloc for CountingAllocator {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    record_allocation();
    unsafe { System.alloc(layout) }
  }

  unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
    record_allocation();
    unsafe { System.alloc_zeroed(layout) }
  }

  unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    record_allocation();
    unsafe { System.realloc(ptr, layout, new_size) }
  }

  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    unsafe { System.dealloc(ptr, layout) }
  }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

/// Runs `f` and returns its result with the number of allocations the current
/// thread made meanwhile.
fn allocations_during<R>(f: impl FnOnce() -> R) -> (R, usize) {
  let before = ALLOCATIONS.get();
  let value = f();
  (value, ALLOCATIONS.get() - before)
}

/// Deserializes `text` from a JSON string that needs no unescaping, so
/// `serde_json` lends a slice of its input and allocates nothing itself.
fn assert_allocations<T>(text: &str, expected: usize, view: fn(&T) -> (&str, bool))
where
  T: DeserializeOwned,
{
  let json = format!("\"{text}\"");

  let (value, allocations) = allocations_during(|| serde_json::from_str::<T>(&json).unwrap());

  let (decoded, is_heap) = view(&value);
  assert_eq!(
    allocations,
    expected,
    "allocations for {} bytes",
    text.len()
  );
  assert_eq!(decoded, text);
  assert_eq!(is_heap, text.len() > INLINE_CAP, "{} bytes", text.len());
}

#[test]
fn deserializing_utf8_bytes_allocates_only_for_heap_storage() {
  let ascii = |len: usize| "a".repeat(len);
  // Twenty 3-byte scalars and two ASCII bytes fill the inline capacity
  // exactly; twenty-one 3-byte scalars overshoot it by one byte.
  let multibyte_full_inline = format!("{}ab", "€".repeat(20));
  let multibyte_min_heap = "€".repeat(21);
  assert_eq!(multibyte_full_inline.len(), INLINE_CAP);
  assert_eq!(multibyte_min_heap.len(), INLINE_CAP + 1);

  let texts = [
    String::new(),
    ascii(1),
    ascii(5),
    ascii(INLINE_CAP - 1),
    ascii(INLINE_CAP),
    multibyte_full_inline,
    ascii(INLINE_CAP + 1),
    ascii(100),
    ascii(1000),
    multibyte_min_heap,
  ];

  for text in &texts {
    // Inline storage costs no allocation; heap storage costs exactly one, the
    // heap buffer itself.
    let expected = usize::from(text.len() > INLINE_CAP);
    assert_allocations::<shared::Utf8Bytes>(text, expected, |v| (v.as_str(), v.is_heap()));
    assert_allocations::<compact::Utf8Bytes>(text, expected, |v| (v.as_str(), v.is_heap()));
  }
}
