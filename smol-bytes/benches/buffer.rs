use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use smol_bytes::Buffer;

fn buffer(c: &mut Criterion) {
  for size in [0, 1, 61, 62] {
    let data = vec![7_u8; size];
    c.bench_function(&format!("buffer/from/{size}"), |bench| {
      bench.iter(|| black_box(Buffer::try_from(black_box(data.as_slice())).unwrap()))
    });
  }
}

criterion_group!(benches, buffer);
criterion_main!(benches);
