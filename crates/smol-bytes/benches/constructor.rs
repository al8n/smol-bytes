use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use smol_bytes::{compact, shared};
use std::hint::black_box;

/// Benchmark constructor performance for different data sizes (from Vec)
fn constructor_from_vec_benchmarks(c: &mut Criterion) {
  // Test various sizes: small (inline), boundary, and large
  let sizes = vec![
    ("8 bytes", 8),
    ("16 bytes", 16),
    ("32 bytes", 32),
    ("62 bytes", 62), // Exactly at inline capacity
    ("64 bytes", 64),
    ("128 bytes", 128),
    ("256 bytes", 256),
    ("512 bytes", 512),
    ("1024 bytes", 1024),
    ("4096 bytes", 4096),
  ];

  for (name, size) in sizes {
    let mut group = c.benchmark_group(format!("constructor/from_vec/{}", name));
    group.throughput(Throughput::Bytes(size as u64));

    let data = vec![0u8; size];

    // Benchmark bytes::Bytes::from(Vec)
    group.bench_function("bytes::Bytes", |b| {
      b.iter(|| {
        let bytes = bytes::Bytes::from(black_box(data.clone()));
        black_box(bytes);
      });
    });

    // Benchmark SmolBytes (Shared) from Vec
    group.bench_function("SmolBytes (Shared)", |b| {
      b.iter(|| {
        let smol = shared::SmolBytes::from(black_box(data.clone()));
        black_box(smol);
      });
    });

    // Benchmark SmolBytes (Compact) from Vec
    group.bench_function("SmolBytes (Compact)", |b| {
      b.iter(|| {
        let smol = compact::SmolBytes::from(black_box(data.clone()));
        black_box(smol);
      });
    });

    group.finish();
  }
}

/// Benchmark constructor performance from slice (copy required)
fn constructor_from_slice_benchmarks(c: &mut Criterion) {
  let sizes = vec![
    ("8 bytes", 8),
    ("16 bytes", 16),
    ("32 bytes", 32),
    ("62 bytes", 62),
    ("64 bytes", 64),
    ("128 bytes", 128),
    ("256 bytes", 256),
    ("512 bytes", 512),
    ("1024 bytes", 1024),
  ];

  for (name, size) in sizes {
    let mut group = c.benchmark_group(format!("constructor/from_slice/{}", name));
    group.throughput(Throughput::Bytes(size as u64));

    let data = vec![0u8; size];

    // Benchmark bytes::Bytes::copy_from_slice
    group.bench_function("bytes::Bytes", |b| {
      b.iter(|| {
        let bytes = bytes::Bytes::copy_from_slice(black_box(&data));
        black_box(bytes);
      });
    });

    // Benchmark SmolBytes (Shared) from slice
    group.bench_function("SmolBytes (Shared)", |b| {
      b.iter(|| {
        let smol = shared::SmolBytes::from(black_box(data.as_slice()));
        black_box(smol);
      });
    });

    // Benchmark SmolBytes (Compact) from slice
    group.bench_function("SmolBytes (Compact)", |b| {
      b.iter(|| {
        let smol = compact::SmolBytes::from(black_box(data.as_slice()));
        black_box(smol);
      });
    });

    group.finish();
  }
}

/// Benchmark constructor performance specifically for inline sizes (≤62 bytes)
fn constructor_inline_benchmarks(c: &mut Criterion) {
  let mut group = c.benchmark_group("constructor/inline_only");

  // Test sizes that are guaranteed to be inline (≤62 bytes)
  let inline_sizes = vec![4, 8, 16, 24, 32, 48, 62];

  for size in inline_sizes {
    let data = vec![0u8; size];

    // From Vec
    group.bench_with_input(
      BenchmarkId::new("bytes::Bytes/from_vec", size),
      &data,
      |b, data| {
        b.iter(|| {
          let bytes = bytes::Bytes::from(black_box(data.clone()));
          black_box(bytes);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Shared)/from_vec", size),
      &data,
      |b, data| {
        b.iter(|| {
          let smol = shared::SmolBytes::from(black_box(data.clone()));
          black_box(smol);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Compact)/from_vec", size),
      &data,
      |b, data| {
        b.iter(|| {
          let smol = compact::SmolBytes::from(black_box(data.clone()));
          black_box(smol);
        });
      },
    );

    // From slice (copy)
    group.bench_with_input(
      BenchmarkId::new("bytes::Bytes/copy_from_slice", size),
      &data,
      |b, data| {
        b.iter(|| {
          let bytes = bytes::Bytes::copy_from_slice(black_box(data));
          black_box(bytes);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Shared)/from_slice", size),
      &data,
      |b, data| {
        b.iter(|| {
          let smol = shared::SmolBytes::from(black_box(data.as_slice()));
          black_box(smol);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Compact)/from_slice", size),
      &data,
      |b, data| {
        b.iter(|| {
          let smol = compact::SmolBytes::from(black_box(data.as_slice()));
          black_box(smol);
        });
      },
    );
  }

  group.finish();
}

/// Benchmark constructor performance specifically for heap sizes (>62 bytes)
fn constructor_heap_benchmarks(c: &mut Criterion) {
  let mut group = c.benchmark_group("constructor/heap_only");

  // Test sizes that are guaranteed to be heap-allocated (>62 bytes)
  let heap_sizes = vec![64, 128, 256, 512, 1024, 4096];

  for size in heap_sizes {
    let data = vec![0u8; size];

    // From Vec
    group.bench_with_input(
      BenchmarkId::new("bytes::Bytes/from_vec", size),
      &data,
      |b, data| {
        b.iter(|| {
          let bytes = bytes::Bytes::from(black_box(data.clone()));
          black_box(bytes);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Shared)/from_vec", size),
      &data,
      |b, data| {
        b.iter(|| {
          let smol = shared::SmolBytes::from(black_box(data.clone()));
          black_box(smol);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Compact)/from_vec", size),
      &data,
      |b, data| {
        b.iter(|| {
          let smol = compact::SmolBytes::from(black_box(data.clone()));
          black_box(smol);
        });
      },
    );

    // From slice (copy and allocate)
    group.bench_with_input(
      BenchmarkId::new("bytes::Bytes/copy_from_slice", size),
      &data,
      |b, data| {
        b.iter(|| {
          let bytes = bytes::Bytes::copy_from_slice(black_box(data));
          black_box(bytes);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Shared)/from_slice", size),
      &data,
      |b, data| {
        b.iter(|| {
          let smol = shared::SmolBytes::from(black_box(data.as_slice()));
          black_box(smol);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Compact)/from_slice", size),
      &data,
      |b, data| {
        b.iter(|| {
          let smol = compact::SmolBytes::from(black_box(data.as_slice()));
          black_box(smol);
        });
      },
    );
  }

  group.finish();
}

/// Benchmark from_static for static data (zero-copy for supported types)
fn constructor_from_static_benchmarks(c: &mut Criterion) {
  let mut group = c.benchmark_group("constructor/from_static");

  // Test various static sizes
  let sizes = vec![
    ("8 bytes", &b"12345678"[..]),
    ("16 bytes", &b"1234567890123456"[..]),
    ("32 bytes", &b"12345678901234567890123456789012"[..]),
    (
      "62 bytes",
      &b"12345678901234567890123456789012345678901234567890123456789012"[..],
    ),
  ];

  for (name, data) in sizes {
    group.bench_function(format!("bytes::Bytes/{}", name), |b| {
      b.iter(|| {
        let bytes = bytes::Bytes::from_static(black_box(data));
        black_box(bytes);
      });
    });

    group.bench_function(format!("SmolBytes (Shared)/{}", name), |b| {
      b.iter(|| {
        let smol = shared::SmolBytes::from_static(black_box(data));
        black_box(smol);
      });
    });

    group.bench_function(format!("SmolBytes (Compact)/{}", name), |b| {
      b.iter(|| {
        let smol = compact::SmolBytes::from_static(black_box(data));
        black_box(smol);
      });
    });
  }

  group.finish();
}

/// Benchmark creating empty buffers
fn constructor_empty_benchmarks(c: &mut Criterion) {
  let mut group = c.benchmark_group("constructor/empty");

  group.bench_function("bytes::Bytes::new", |b| {
    b.iter(|| {
      let bytes = bytes::Bytes::new();
      black_box(bytes);
    });
  });

  group.bench_function("SmolBytes (Shared)::new", |b| {
    b.iter(|| {
      let smol = shared::SmolBytes::new();
      black_box(smol);
    });
  });

  group.bench_function("SmolBytes (Compact)::new", |b| {
    b.iter(|| {
      let smol = compact::SmolBytes::new();
      black_box(smol);
    });
  });

  group.finish();
}

criterion_group! {
  name = constructor_group;
  config = Criterion::default().configure_from_args();
  targets =
    constructor_from_vec_benchmarks,
    constructor_from_slice_benchmarks,
    constructor_inline_benchmarks,
    constructor_heap_benchmarks,
    constructor_from_static_benchmarks,
    constructor_empty_benchmarks
}
criterion_main!(constructor_group);
