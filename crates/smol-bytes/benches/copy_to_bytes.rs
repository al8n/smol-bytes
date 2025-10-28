use bytes::Buf;
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use smol_bytes::strategy::{compact, shared};
use std::hint::black_box;

fn copy_to_bytes_benchmarks(c: &mut Criterion) {
  let cases = [
    ("inline", 32usize, 16usize),
    ("boundary", 62, 31),
    ("heap", 256, 128),
  ];

  let mut group = c.benchmark_group("copy_to_bytes");

  for (label, size, chunk) in cases {
    let bytes_template = bytes::Bytes::from(vec![0u8; size]);
    let shared_template = shared::SmolBytes::from(vec![0u8; size]);
    let compact_template = compact::SmolBytes::from(vec![0u8; size]);

    group.bench_with_input(BenchmarkId::new("bytes::Bytes", label), &size, |b, _| {
      b.iter_batched(
        || bytes_template.clone(),
        |mut data| {
          let result = data.copy_to_bytes(chunk);
          black_box(result);
          black_box(&data);
        },
        BatchSize::SmallInput,
      );
    });

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Shared)", label),
      &size,
      |b, _| {
        b.iter_batched(
          || shared_template.clone(),
          |mut data| {
            let result = data.copy_to_bytes(chunk);
            black_box(result);
            black_box(&data);
          },
          BatchSize::SmallInput,
        );
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Compact)", label),
      &size,
      |b, _| {
        b.iter_batched(
          || compact_template.clone(),
          |mut data| {
            let result = data.copy_to_bytes(chunk);
            black_box(result);
            black_box(&data);
          },
          BatchSize::SmallInput,
        );
      },
    );
  }

  group.finish();
}

criterion_group! {
  name = copy_to_bytes_group;
  config = Criterion::default().configure_from_args();
  targets = copy_to_bytes_benchmarks
}
criterion_main!(copy_to_bytes_group);
