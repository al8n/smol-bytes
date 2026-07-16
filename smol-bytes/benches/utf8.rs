use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use smol_bytes::Utf8BytesMut;

fn utf8(c: &mut Criterion) {
  for value in [
    String::new(),
    "a".into(),
    "a".repeat(61),
    "a".repeat(62),
    "a".repeat(63),
  ] {
    c.bench_function(&format!("utf8/freeze/{}", value.len()), |bench| {
      bench.iter_batched(
        || Utf8BytesMut::from(black_box(value.as_str())),
        |value| black_box(value.freeze_shared()),
        BatchSize::SmallInput,
      )
    });
  }
}

criterion_group!(benches, utf8);
criterion_main!(benches);
