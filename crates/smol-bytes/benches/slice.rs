use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use smol_bytes::strategy::{compact, shared};
use std::hint::black_box;

struct SliceCase {
  label: &'static str,
  total: usize,
  range: (usize, usize),
}

fn slice_benchmarks(c: &mut Criterion) {
  let cases = [
    // Original inline; slice remains inline.
    SliceCase {
      label: "inline_src_inline_slice",
      total: 40,
      range: (8, 32),
    },
    // Original heap; slice taken from prefix and becomes inline.
    SliceCase {
      label: "heap_src_prefix_inline_slice",
      total: 120,
      range: (0, 40),
    },
    // Original heap; slice taken from suffix and becomes inline.
    SliceCase {
      label: "heap_src_suffix_inline_slice",
      total: 120,
      range: (80, 120),
    },
    // Original heap; slice straddles middle and exceeds inline capacity, staying heap.
    SliceCase {
      label: "heap_src_middle_heap_slice",
      total: 128,
      range: (32, 112),
    },
    // Original heap; wide slice retains heap allocation.
    SliceCase {
      label: "heap_src_wide_heap_slice",
      total: 256,
      range: (16, 208),
    },
  ];

  let mut group = c.benchmark_group("slice");

  for case in &cases {
    let bytes_template = bytes::Bytes::from(vec![0u8; case.total]);
    let shared_template = shared::SmolBytes::from(vec![0u8; case.total]);
    let compact_template = compact::SmolBytes::from(vec![0u8; case.total]);

    group.bench_with_input(
      BenchmarkId::new("bytes::Bytes", case.label),
      &case.total,
      |b, _| {
        b.iter(|| {
          let slice = bytes_template.slice(case.range.0..case.range.1);
          black_box(slice);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Shared)", case.label),
      &case.total,
      |b, _| {
        b.iter(|| {
          let slice = shared_template.slice(case.range.0..case.range.1);
          black_box(slice);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Compact)", case.label),
      &case.total,
      |b, _| {
        b.iter(|| {
          let slice = compact_template.slice(case.range.0..case.range.1);
          black_box(slice);
        });
      },
    );
  }

  group.finish();
}

criterion_group! {
  name = slice_group;
  config = Criterion::default().configure_from_args();
  targets = slice_benchmarks
}
criterion_main!(slice_group);
