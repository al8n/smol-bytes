# Smol Bytes Benchmarks (Quick Run)

Benchmarks were executed with Criterion's quick mode (`cargo bench --bench <name> -- --quick`) using the plotters backend (GNUPlot unavailable on this host). Each benchmark compares `bytes::Bytes`, `strategy::shared::SmolBytes`, and `strategy::compact::SmolBytes` across scenarios that exercise:

- Fully inline buffers,
- Heap buffers that shrink to inline pieces,
- Mixed inline/heap splits, and
- Fully heap-resident results.

Reported timings are the median (ns) from Criterion's quick run.

## `advance`

| Case | bytes::Bytes | SmolBytes (Shared) | SmolBytes (Compact) |
| --- | --- | --- | --- |
| Inline | 7.36 ns | **2.72 ns** | 2.86 ns |
| Boundary | 7.29 ns | **2.25 ns** | 2.82 ns |
| Heap | 7.23 ns | 7.20 ns | **7.17 ns** |

## `split_to`

| Case | bytes::Bytes | SmolBytes (Shared) | SmolBytes (Compact) |
| --- | --- | --- | --- |
| Inline src → inline parts | 17.89 ns | **8.91 ns** | 9.03 ns |
| Heap src → two inline parts | 20.91 ns | **12.07 ns** | 15.17 ns |
| Heap src → prefix inline, remainder heap | 17.88 ns | **12.27 ns** | 12.35 ns |
| Heap src → prefix heap, remainder inline | **17.52 ns** | 22.10 ns | 21.91 ns |
| Heap src → two heap parts | **18.02 ns** | 22.78 ns | 22.72 ns |

## `split_off`

| Case | bytes::Bytes | SmolBytes (Shared) | SmolBytes (Compact) |
| --- | --- | --- | --- |
| Inline src → inline parts | 17.88 ns | 8.95 ns | **8.94 ns** |
| Heap src → two inline parts | 17.59 ns | **11.15 ns** | 15.07 ns |
| Heap head heap / tail inline | 17.41 ns | **10.89 ns** | 11.39 ns |
| Heap head inline / tail heap | **17.65 ns** | 21.83 ns | 21.88 ns |
| Heap src → two heap parts | **17.42 ns** | 21.53 ns | 22.70 ns |

## `copy_to_bytes`

| Case | bytes::Bytes | SmolBytes (Shared) | SmolBytes (Compact) |
| --- | --- | --- | --- |
| Inline | **18.07 ns** | 29.24 ns | 29.36 ns |
| Boundary | **17.92 ns** | 37.56 ns | 37.70 ns |
| Heap | **17.87 ns** | 22.90 ns | 23.35 ns |

## `slice`

| Case | bytes::Bytes | SmolBytes (Shared) | SmolBytes (Compact) |
| --- | --- | --- | --- |
| Inline src → inline slice | 9.81 ns | **5.06 ns** | 5.08 ns |
| Heap src → inline prefix | 9.82 ns | 13.73 ns | **5.68 ns** |
| Heap src → inline suffix | 9.64 ns | 13.75 ns | **5.70 ns** |
| Heap src → mid-range heap slice | **9.54 ns** | 13.72 ns | 13.87 ns |
| Heap src → wide heap slice | **9.61 ns** | 13.67 ns | 13.72 ns |

### Observations

- Shared and compact strategies substantially reduce inline `advance` costs versus `bytes::Bytes`, with the shared strategy marginally ahead.
- For `split_to`/`split_off`, `SmolBytes` excels when the resulting parts become inline, but `bytes::Bytes` remains superior when both sides stay heap-resident (zero-copy slicing).
- `copy_to_bytes` favours `bytes::Bytes` regardless of size because both SmolBytes strategies must copy into a new `Bytes`.
- `slice` shows compact strategy advantages when converting heap data into inline slices, while `bytes::Bytes` still leads for heap-sized slices.

All measurements stem from the current checkout and quick benchmark runs; full-duration runs may shift rankings slightly. Continuous benchmark artifacts reside under `target/criterion`.
