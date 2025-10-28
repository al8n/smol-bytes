use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use smol_bytes::strategy::{compact, shared};
use std::hint::black_box;

struct SplitCase {
  label: &'static str,
  total: usize,
  split_at: usize,
}

fn split_off_benchmarks(c: &mut Criterion) {
  let cases = [
    // Original buffer is inline; both resulting segments remain inline.
    SplitCase {
      label: "inline_src_inline_segments",
      total: 40,
      split_at: 22,
    },
    // Original buffer is heap; both segments become inline after split.
    SplitCase {
      label: "heap_src_two_inline_segments",
      total: 80,
      split_at: 40,
    },
    // Original buffer is heap; tail converts inline, head remains heap.
    SplitCase {
      label: "heap_src_head_heap_tail_inline",
      total: 128,
      split_at: 96,
    },
    // Original buffer is heap; tail stays heap, head converts inline.
    SplitCase {
      label: "heap_src_head_inline_tail_heap",
      total: 128,
      split_at: 48,
    },
    // Original buffer is heap; both segments remain heap.
    SplitCase {
      label: "heap_src_two_heap_segments",
      total: 256,
      split_at: 128,
    },
  ];

  let mut group = c.benchmark_group("split_off");

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
            let tail = data.split_off(case.split_at);
            black_box(tail);
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
            let tail = data.split_off(case.split_at);
            black_box(tail);
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
            let tail = data.split_off(case.split_at);
            black_box(tail);
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
  name = split_off_group;
  config = Criterion::default().configure_from_args();
  targets = split_off_benchmarks
}
criterion_main!(split_off_group);
