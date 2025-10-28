use bytes::Buf;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use smol_bytes::strategy::{compact, shared};
use std::hint::black_box;

/// Benchmark clone performance for different data sizes
fn clone_benchmarks(c: &mut Criterion) {
  // Test various sizes: small (inline), boundary, and large
  let sizes = vec![
    ("8 bytes", 8),
    ("16 bytes", 16),
    ("32 bytes", 32),
    ("62 bytes", 62), // Exactly at inline capacity
    ("64 bytes", 64),
    ("128 bytes", 128),
  ];

  for (name, size) in sizes {
    let mut group = c.benchmark_group(format!("clone/{}", name));
    group.throughput(Throughput::Bytes(size as u64));

    let data = vec![0u8; size];

    // Benchmark bytes::Bytes clone
    group.bench_function("bytes::Bytes", |b| {
      let bytes = bytes::Bytes::from(data.clone());
      b.iter(|| {
        let cloned = black_box(bytes.clone());
        black_box(cloned);
      });
    });

    // Benchmark SmolBytes (Shared strategy) clone
    group.bench_function("SmolBytes (Shared)", |b| {
      let smol = shared::SmolBytes::from(data.clone());
      b.iter(|| {
        let cloned = black_box(smol.clone());
        black_box(cloned);
      });
    });

    // Benchmark SmolBytes (Compact strategy) clone
    group.bench_function("SmolBytes (Compact)", |b| {
      let smol = compact::SmolBytes::from(data.clone());
      b.iter(|| {
        let cloned = black_box(smol.clone());
        black_box(cloned);
      });
    });

    group.finish();
  }
}

/// Benchmark clone performance specifically for inline data
fn inline_clone_benchmarks(c: &mut Criterion) {
  let mut group = c.benchmark_group("clone/inline_only");

  // Test sizes that are guaranteed to be inline (≤62 bytes)
  let inline_sizes = vec![4, 8, 16, 24, 32, 38];

  for size in inline_sizes {
    let data = vec![0u8; size];

    group.bench_with_input(BenchmarkId::new("bytes::Bytes", size), &data, |b, data| {
      let bytes = bytes::Bytes::from(data.clone());
      b.iter(|| {
        let cloned = black_box(bytes.clone());
        black_box(cloned);
      });
    });

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Shared)", size),
      &data,
      |b, data| {
        let smol = shared::SmolBytes::from(data.clone());
        b.iter(|| {
          let cloned = black_box(smol.clone());
          black_box(cloned);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Compact)", size),
      &data,
      |b, data| {
        let smol = compact::SmolBytes::from(data.clone());
        b.iter(|| {
          let cloned = black_box(smol.clone());
          black_box(cloned);
        });
      },
    );
  }

  group.finish();
}

/// Benchmark clone performance specifically for heap-allocated data
fn heap_clone_benchmarks(c: &mut Criterion) {
  let mut group = c.benchmark_group("clone/heap_only");

  // Test sizes that are guaranteed to be heap-allocated (>62 bytes)
  let heap_sizes = vec![64, 128, 256, 512, 1024, 4096];

  for size in heap_sizes {
    let data = vec![0u8; size];

    group.bench_with_input(BenchmarkId::new("bytes::Bytes", size), &data, |b, data| {
      let bytes = bytes::Bytes::from(data.clone());
      b.iter(|| {
        let cloned = black_box(bytes.clone());
        black_box(cloned);
      });
    });

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Shared)", size),
      &data,
      |b, data| {
        let smol = shared::SmolBytes::from(data.clone());
        b.iter(|| {
          let cloned = black_box(smol.clone());
          black_box(cloned);
        });
      },
    );

    group.bench_with_input(
      BenchmarkId::new("SmolBytes (Compact)", size),
      &data,
      |b, data| {
        let smol = compact::SmolBytes::from(data.clone());
        b.iter(|| {
          let cloned = black_box(smol.clone());
          black_box(cloned);
        });
      },
    );
  }

  group.finish();
}

/// Benchmark clone performance after operations (to test inline conversion effects)
fn clone_after_advance_benchmarks(c: &mut Criterion) {
  let mut group = c.benchmark_group("clone/after_advance");

  // Start with 100 bytes, advance by 70, leaving 30 bytes
  // Shared: stays heap
  // Compact: converts to inline
  let data = vec![0u8; 100];

  group.bench_function("bytes::Bytes (70 advanced, 30 remaining)", |b| {
    b.iter(|| {
      let mut bytes = bytes::Bytes::from(data.clone());
      bytes.advance(70);
      let cloned = black_box(bytes.clone());
      black_box(cloned);
    });
  });

  group.bench_function("SmolBytes Shared (70 advanced, 30 remaining)", |b| {
    b.iter(|| {
      let mut smol = shared::SmolBytes::from(data.clone());
      smol.advance(70);
      // Still heap-allocated, should be as fast as bytes::Bytes
      let cloned = black_box(smol.clone());
      black_box(cloned);
    });
  });

  group.bench_function("SmolBytes Compact (70 advanced, 30 remaining)", |b| {
    b.iter(|| {
      let mut smol = compact::SmolBytes::from(data.clone());
      smol.advance(70);
      // Converted to inline, should be slower due to memcpy
      let cloned = black_box(smol.clone());
      black_box(cloned);
    });
  });

  group.finish();
}

/// Benchmark multiple sequential clones (typical sharing pattern)
fn sequential_clone_benchmarks(c: &mut Criterion) {
  let mut group = c.benchmark_group("clone/sequential_10_clones");

  let sizes = vec![("32 bytes (inline)", 32), ("1024 bytes (heap)", 1024)];

  for (name, size) in sizes {
    let data = vec![0u8; size];

    group.bench_function(format!("bytes::Bytes/{}", name), |b| {
      let bytes = bytes::Bytes::from(data.clone());
      b.iter(|| {
        let mut clones = Vec::with_capacity(10);
        for _ in 0..10 {
          clones.push(bytes.clone());
        }
        black_box(clones);
      });
    });

    group.bench_function(format!("SmolBytes (Shared)/{}", name), |b| {
      let smol = shared::SmolBytes::from(data.clone());
      b.iter(|| {
        let mut clones = Vec::with_capacity(10);
        for _ in 0..10 {
          clones.push(smol.clone());
        }
        black_box(clones);
      });
    });

    group.bench_function(format!("SmolBytes (Compact)/{}", name), |b| {
      let smol = compact::SmolBytes::from(data.clone());
      b.iter(|| {
        let mut clones = Vec::with_capacity(10);
        for _ in 0..10 {
          clones.push(smol.clone());
        }
        black_box(clones);
      });
    });
  }

  group.finish();
}

criterion_group! {
  name = clone_group;
  config = Criterion::default().configure_from_args();
  targets =
    clone_benchmarks,
    inline_clone_benchmarks,
    heap_clone_benchmarks,
    clone_after_advance_benchmarks,
    sequential_clone_benchmarks
}
criterion_main!(clone_group);
