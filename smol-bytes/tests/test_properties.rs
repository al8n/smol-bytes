#![warn(rust_2018_idioms)]

//! Compact state-machine properties for representation transitions and UTF-8
//! boundary failures. The fixed examples in the integration suite cover exact
//! regressions; these compare short operation sequences to standard types.

use proptest::prelude::*;
use smol_bytes::{Buf, Buffer, BytesMut, INLINE_CAP, Utf8BytesMut};

fn bytes_cases() -> impl Strategy<Value = Vec<u8>> {
  prop_oneof![
    Just(Vec::new()),
    Just(vec![0]),
    Just(vec![0; 61]),
    Just(vec![0; 62]),
    Just(vec![0; 63]),
    prop::collection::vec(any::<u8>(), 0..96),
  ]
}

fn utf8_cases() -> impl Strategy<Value = String> {
  prop::collection::vec(0_u8..6, 0..48).prop_map(|chars| {
    chars
      .into_iter()
      .map(|index| match index {
        0 => 'a',
        1 => 'é',
        2 => '€',
        3 => '🦀',
        4 => 'e',
        _ => '\u{301}',
      })
      .collect()
  })
}

fn boundaries(value: &str) -> Vec<usize> {
  let mut boundaries: Vec<_> = value.char_indices().map(|(index, _)| index).collect();
  boundaries.push(value.len());
  boundaries
}

proptest! {
  #![proptest_config(ProptestConfig::with_cases(48))]

  #[test]
  fn buffer_tracks_a_vec_through_small_sequences(
    source in bytes_cases(),
    operations in prop::collection::vec((0_u8..5, any::<u8>()), 0..32),
  ) {
    let mut expected = source[..source.len().min(INLINE_CAP)].to_vec();
    let mut actual = Buffer::try_from(expected.as_slice()).expect("bounded by inline capacity");

    for (operation, argument) in operations {
      match operation {
        0 if actual.remaining_mut() > 0 => {
          actual.put_u8(argument);
          expected.push(argument);
        }
        1 => {
          let count = usize::from(argument) % (expected.len() + 1);
          actual.advance(count);
          expected.drain(..count);
        }
        2 => {
          let length = usize::from(argument) % (expected.len() + 4);
          actual.truncate(length);
          expected.truncate(length);
        }
        3 => {
          let at = usize::from(argument) % (expected.len() + 1);
          let head = actual.try_split_to(at).expect("bounded split");
          prop_assert_eq!(head.as_slice(), &expected[..at]);
          expected.drain(..at);
        }
        _ => {
          let start = usize::from(argument) % (expected.len() + 1);
          let end = start + (usize::from(argument.rotate_left(3)) % (expected.len() - start + 1));
          let slice = actual.try_slice(start..end).unwrap();
          prop_assert_eq!(slice.as_slice(), &expected[start..end]);
        }
      }

      prop_assert_eq!(actual.as_slice(), expected.as_slice());
    }
  }

  #[test]
  fn bytes_mut_tracks_a_vec_across_promotion_split_and_freeze(
    source in bytes_cases(),
    operations in prop::collection::vec((0_u8..6, any::<u8>()), 0..32),
  ) {
    let mut expected = source;
    let mut actual = BytesMut::from(expected.as_slice());

    for (operation, argument) in operations {
      match operation {
        0 => {
          let extension = vec![argument; usize::from(argument % 4)];
          actual.extend_from_slice(&extension);
          expected.extend_from_slice(&extension);
        }
        1 => {
          let count = usize::from(argument) % (expected.len() + 1);
          actual.advance(count);
          expected.drain(..count);
        }
        2 => {
          let length = usize::from(argument) % (expected.len() + 4);
          actual.truncate(length);
          expected.truncate(length);
        }
        3 => {
          let at = usize::from(argument) % (expected.len() + 1);
          match actual.try_split_to(at).expect("bounded split") {
            Ok(head) => prop_assert_eq!(head.as_slice(), &expected[..at]),
            Err(head) => prop_assert_eq!(head.as_slice(), &expected[..at]),
          }
          expected.drain(..at);
        }
        4 => {
          let additional = usize::from(argument) + INLINE_CAP + 1;
          actual.reserve(additional);
          prop_assert!(actual.capacity() >= actual.len() + additional);
          prop_assert!(actual.is_heap());
        }
        _ => {
          let shared = actual.clone().freeze_shared();
          let compact = actual.clone().freeze_compact();
          prop_assert_eq!(shared.as_ref(), expected.as_slice());
          prop_assert_eq!(compact.as_ref(), expected.as_slice());
        }
      }

      prop_assert_eq!(actual.as_slice(), expected.as_slice());
    }
  }

  #[test]
  fn immutable_strategies_slice_like_bytes(
    source in bytes_cases(),
    start_seed in any::<u8>(),
    end_seed in any::<u8>(),
  ) {
    let shared = BytesMut::from(source.as_slice()).freeze_shared();
    let compact = BytesMut::from(source.as_slice()).freeze_compact();
    let native = bytes::Bytes::copy_from_slice(&source);
    let start = usize::from(start_seed) % (source.len() + 1);
    let end = start + (usize::from(end_seed) % (source.len() - start + 1));

    let shared_slice = shared.slice(start..end);
    let compact_slice = compact.slice(start..end);
    let native_slice = native.slice(start..end);
    prop_assert_eq!(shared_slice.as_ref(), native_slice.as_ref());
    prop_assert_eq!(compact_slice.as_ref(), native_slice.as_ref());
    if source.len() > INLINE_CAP {
      prop_assert!(shared.slice(0..1).is_heap());
    }
  }

  #[test]
  fn mutable_utf8_tracks_string_and_rejects_invalid_truncation(
    source in utf8_cases(),
    operations in prop::collection::vec((0_u8..5, any::<u8>()), 0..32),
  ) {
    let mut expected = source;
    let mut actual = Utf8BytesMut::from(expected.as_str());

    for (operation, argument) in operations {
      match operation {
        0 => {
          let character = match argument % 4 {
            0 => 'a',
            1 => 'é',
            2 => '€',
            _ => '🦀',
          };
          actual.push(character);
          expected.push(character);
        }
        1 => {
          let length = usize::from(argument) % (expected.len() + 4);
          let before = expected.clone();
          let result = actual.try_truncate(length);
          if length >= before.len() || before.is_char_boundary(length) {
            prop_assert!(result.is_ok());
            expected.truncate(length);
          } else {
            prop_assert!(result.is_err());
            prop_assert_eq!(actual.as_str(), before.as_str());
          }
        }
        2 => {
          let positions = boundaries(&expected);
          let at = positions[usize::from(argument) % positions.len()];
          let head = actual.try_split_to(at).expect("valid boundary");
          prop_assert_eq!(head.as_str(), &expected[..at]);
          expected = expected[at..].to_owned();
        }
        3 => {
          let positions = boundaries(&expected);
          let start = positions[usize::from(argument) % positions.len()];
          let end = positions[usize::from(argument.rotate_left(2)) % positions.len()];
          let (start, end) = if start <= end { (start, end) } else { (end, start) };
          let slice = actual.try_slice(start..end).unwrap();
          prop_assert_eq!(slice.as_str(), &expected[start..end]);
        }
        _ => actual.reserve(usize::from(argument) + INLINE_CAP + 1),
      }

      prop_assert_eq!(actual.as_str(), expected.as_str());
    }

    let shared = actual.clone().freeze_shared();
    let compact = actual.freeze_compact();
    prop_assert_eq!(shared.as_str(), expected.as_str());
    prop_assert_eq!(compact.as_str(), expected.as_str());
  }
}
