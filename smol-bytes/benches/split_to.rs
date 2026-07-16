use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use smol_bytes::{compact, shared};
use std::hint::black_box;

struct SplitCase {
  label: &'static str,
  total: usize,
  split_at: usize,
}

fn split_to_benchmarks(c: &mut Criterion) {
  let cases = [
    // Inline source split near its midpoint.
    SplitCase {
      label: "inline_source_midpoint",
      total: 40,
      split_at: 18,
    },
    // Heap source split evenly.
    SplitCase {
      label: "heap_source_even_split",
      total: 80,
      split_at: 40,
    },
    // Heap source with a short prefix.
    SplitCase {
      label: "heap_source_short_prefix",
      total: 128,
      split_at: 48,
    },
    // Heap source with a short remainder.
    SplitCase {
      label: "heap_source_short_remainder",
      total: 128,
      split_at: 96,
    },
    // Large heap source split evenly.
    SplitCase {
      label: "large_heap_source_even_split",
      total: 256,
      split_at: 128,
    },
  ];

  let mut group = c.benchmark_group("split_to");

  for case in &cases {
    let bytes_template = bytes::Bytes::from(vec![0u8; case.total]);
    let shared_template = shared::Bytes::from(vec![0u8; case.total]);
    let compact_template = compact::Bytes::from(vec![0u8; case.total]);

    group.bench_with_input(
      BenchmarkId::new("bytes::Bytes", case.label),
      &case.total,
      |b, _| {
        b.iter_batched(
          || bytes_template.clone(),
          |mut data| {
            let prefix = data.split_to(case.split_at);
            black_box(prefix);
            black_box(data);
          },
          BatchSize::SmallInput,
        );
      },
    );

    group.bench_with_input(
      BenchmarkId::new("Bytes (Shared)", case.label),
      &case.total,
      |b, _| {
        b.iter_batched(
          || shared_template.clone(),
          |mut data| {
            let prefix = data.split_to(case.split_at);
            black_box(prefix);
            black_box(data);
          },
          BatchSize::SmallInput,
        );
      },
    );

    group.bench_with_input(
      BenchmarkId::new("Bytes (Compact)", case.label),
      &case.total,
      |b, _| {
        b.iter_batched(
          || compact_template.clone(),
          |mut data| {
            let prefix = data.split_to(case.split_at);
            black_box(prefix);
            black_box(data);
          },
          BatchSize::SmallInput,
        );
      },
    );
  }

  group.finish();
}

criterion_group! {
  name = split_to_group;
  config = Criterion::default().configure_from_args();
  targets = split_to_benchmarks
}
criterion_main!(split_to_group);
