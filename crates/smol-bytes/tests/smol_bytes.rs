use smol_bytes::{shared::SmolBytes, INLINE_CAP};
use std::sync::Arc;

#[test]
fn inline_construction() {
  let data = b"abc";
  let smol: SmolBytes = SmolBytes::copy_from_slice(data);
  assert_eq!(smol.as_slice(), data);
  assert!(!smol.is_heap());
}

#[test]
fn static_construction() {
  const DATA: &[u8] = b"static-bytes";
  const SMOL: SmolBytes = SmolBytes::new_inline(DATA);
  assert_eq!(SMOL.as_slice(), DATA);
  assert!(!SMOL.is_heap());
}

#[test]
fn heap_construction() {
  let data = vec![7u8; INLINE_CAP + 1];
  let smol: SmolBytes = SmolBytes::copy_from_slice(&data);
  assert_eq!(smol.as_slice(), data.as_slice());
  assert!(smol.is_heap());
}

#[test]
fn arc_roundtrip() {
  let arc: Arc<[u8]> = Arc::from(&b"hello world"[..]);
  let smol: SmolBytes = SmolBytes::from(arc.clone());
  assert_eq!(smol.as_slice(), &b"hello world"[..]);
  let arc2: Arc<[u8]> = smol.clone().into();
  assert_eq!(Arc::strong_count(&arc2), 1);
}

// #[test]
// fn builder_collects_inline() {
//   let mut builder = SmolBytesBuilder::new();
//   builder.extend_copy_from_slice(b"abcd");
//   let smol = builder.finish();
//   assert_eq!(smol.as_slice(), b"abcd");
//   assert!(!smol.is_heap());
// }

// #[test]
// fn builder_promotes_to_heap() {
//   let mut builder = SmolBytesBuilder::new();
//   builder.extend_copy_from_slice(&vec![1u8; INLINE_CAP + 5]);
//   let smol = builder.finish();
//   assert_eq!(smol.len(), INLINE_CAP + 5);
//   assert!(smol.is_heap());
// }

// #[cfg(not(miri))]
// mod proptests {
//   use super::*;
//   use proptest::collection::vec;
//   use proptest::prelude::*;

//   fn check_props(
//     bytes: &[u8],
//     smol: SmolBytes<super::ConversionFriendly>,
//   ) -> Result<(), proptest::test_runner::TestCaseError> {
//     prop_assert_eq!(smol.as_slice(), bytes);
//     prop_assert_eq!(smol.len(), bytes.len());
//     prop_assert_eq!(smol.is_empty(), bytes.is_empty());
//     if bytes.len() <= INLINE_CAP {
//       prop_assert!(!smol.is_heap());
//     } else {
//       prop_assert!(smol.is_heap());
//     }
//     Ok(())
//   }

//   proptest! {
//       #[test]
//       fn roundtrip(data in vec(any::<u8>(), 0..200)) {
//           check_props(&data, SmolBytes::copy_from_slice(&data))?;
//       }

//       #[test]
//       fn from_iter_chunks(chunks in vec(vec(any::<u8>(), 0..64), 1..16)) {
//           let collected: Vec<u8> = chunks.iter().flatten().copied().collect();
//           let smol: SmolBytes<super::ConversionFriendly> = chunks.iter().map(|chunk| chunk.as_slice()).collect();
//           check_props(&collected, smol)?;
//       }
//   }
// }
