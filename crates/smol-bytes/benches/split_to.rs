use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use smol_bytes::strategy::{compact, shared};
use std::hint::black_box;

struct SplitCase {
  label: &'static str,
  total: usize,
  split_at: usize,
}

fn split_to_benchmarks(c: &mut Criterion) {
  let cases = [
    // Original buffer is inline; both resulting segments remain inline.
    SplitCase {
      label: "inline_src_inline_segments",
      total: 40,
      split_at: 18,
    },
    // Original buffer is heap; both resulting segments shrink to inline.
    SplitCase {
      label: "heap_src_two_inline_segments",
      total: 80,
      split_at: 40,
    },
    // Original buffer is heap; prefix becomes inline, remainder stays heap.
    SplitCase {
      label: "heap_src_prefix_inline_remainder_heap",
      total: 128,
      split_at: 48,
    },
    // Original buffer is heap; prefix remains heap, remainder converts inline.
    SplitCase {
      label: "heap_src_prefix_heap_remainder_inline",
      total: 128,
      split_at: 96,
    },
    // Original buffer is heap; both segments remain heap.
    SplitCase {
      label: "heap_src_two_heap_segments",
      total: 256,
      split_at: 128,
    },
  ];

  let mut group = c.benchmark_group("split_to");

  for case in &cases {
    let bytes_template = bytes::Bytes::from(vec![0u8; case.total]);
    let shared_template = shared::SmolBytes::from(vec![0u8; case.total]);
    let compact_template = compact::SmolBytes::from(vec![0u8; case.total]);

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
      BenchmarkId::new("SmolBytes (Shared)", case.label),
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
      BenchmarkId::new("SmolBytes (Compact)", case.label),
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
