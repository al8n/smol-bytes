use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use smol_bytes::{compact, shared};
use std::hint::black_box;

struct SliceCase {
  label: &'static str,
  total: usize,
  range: (usize, usize),
}

fn slice_benchmarks(c: &mut Criterion) {
  let cases = [
    // Inline source with a short interior range.
    SliceCase {
      label: "inline_source_short_range",
      total: 40,
      range: (8, 32),
    },
    // Heap source with a short prefix range.
    SliceCase {
      label: "heap_source_short_prefix",
      total: 120,
      range: (0, 40),
    },
    // Heap source with a short suffix range.
    SliceCase {
      label: "heap_source_short_suffix",
      total: 120,
      range: (80, 120),
    },
    // Heap source with a medium interior range.
    SliceCase {
      label: "heap_source_medium_range",
      total: 128,
      range: (32, 112),
    },
    // Heap source with a wide interior range.
    SliceCase {
      label: "heap_source_wide_range",
      total: 256,
      range: (16, 208),
    },
  ];

  let mut group = c.benchmark_group("slice");

  for case in &cases {
    let bytes_template = bytes::Bytes::from(vec![0u8; case.total]);
    let shared_template = shared::Bytes::from(vec![0u8; case.total]);
    let compact_template = compact::Bytes::from(vec![0u8; case.total]);

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
      BenchmarkId::new("Bytes (Shared)", case.label),
      &case.total,
      |b, _| {
        b.iter(|| {
          let slice = shared_template.slice(case.range.0..case.range.1);
          black_box(slice);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("Bytes (Compact)", case.label),
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
