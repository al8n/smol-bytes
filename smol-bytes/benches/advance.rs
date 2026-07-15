use bytes::Buf;
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use smol_bytes::{compact, shared};
use std::hint::black_box;

fn advance_benchmarks(c: &mut Criterion) {
  let cases = [
    ("inline", 32usize, 16usize),
    ("boundary", 62, 31),
    ("heap", 256, 128),
  ];

  let mut group = c.benchmark_group("advance");

  for (label, size, advance_by) in cases {
    let bytes_template = bytes::Bytes::from(vec![0u8; size]);
    let shared_template = shared::Bytes::from(vec![0u8; size]);
    let compact_template = compact::Bytes::from(vec![0u8; size]);

    group.bench_with_input(BenchmarkId::new("bytes::Bytes", label), &size, |b, _| {
      b.iter_batched(
        || bytes_template.clone(),
        |mut data| {
          data.advance(advance_by);
          black_box(&data);
        },
        BatchSize::SmallInput,
      );
    });

    group.bench_with_input(BenchmarkId::new("Bytes (Shared)", label), &size, |b, _| {
      b.iter_batched(
        || shared_template.clone(),
        |mut data| {
          data.advance(advance_by);
          black_box(&data);
        },
        BatchSize::SmallInput,
      );
    });

    group.bench_with_input(BenchmarkId::new("Bytes (Compact)", label), &size, |b, _| {
      b.iter_batched(
        || compact_template.clone(),
        |mut data| {
          data.advance(advance_by);
          black_box(&data);
        },
        BatchSize::SmallInput,
      );
    });
  }

  group.finish();
}

criterion_group! {
  name = advance_group;
  config = Criterion::default().configure_from_args();
  targets = advance_benchmarks
}
criterion_main!(advance_group);
