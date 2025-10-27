#[test]
fn inline_capacity_matches_constant() {
  assert_eq!(crate::SmolBytes::inline_capacity(), super::INLINE_CAP);
}

#[test]
fn default_is_empty() {
  let smol = crate::SmolBytes::default();
  assert!(smol.is_empty());
  assert_eq!(smol.len(), 0);
}
