use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use smol_bytes::BytesMut;

fn bytes_mut(c: &mut Criterion) {
  for size in [0, 1, 61, 62, 63] {
    c.bench_function(&format!("bytes_mut/append/{size}"), |bench| {
      bench.iter_batched(
        || BytesMut::from(vec![1_u8; size]),
        |mut value| {
          value.extend_from_slice(black_box(b"x"));
          black_box(value)
        },
        BatchSize::SmallInput,
      )
    });
  }
}

criterion_group!(benches, bytes_mut);
criterion_main!(benches);
