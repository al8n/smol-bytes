# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-16

Initial release.

### Added

- Small-buffer-optimized byte buffers that store up to 62 bytes inline and fall
  back to [`bytes`](https://crates.io/crates/bytes)-backed heap storage for
  larger data. Every handle is 64 bytes and `Option<T>` is niche-packed to the
  same size.
- Two immutable strategies: `shared::Bytes` (the default `Bytes`) preserves heap
  storage for zero-copy `bytes::Bytes` interop, and `compact::Bytes` inlines
  heap-backed values that shrink to fit.
- Mutable `BytesMut` with inline-to-heap growth, and the fixed-capacity `Buffer`
  usable in `no_std`.
- UTF-8 wrappers — `Utf8Buffer`, `Utf8Bytes`, and `Utf8BytesMut` — with
  `String`-like, character-boundary-checked split, slice, and truncate
  operations.
- `bytes::Buf` for all types and `bytes::BufMut` for the mutable ones, plus
  zero-copy conversions with `bytes::Bytes` and `bytes::BytesMut`.
- Optional integrations behind features: `serde`, `borsh`, `arbitrary`,
  `quickcheck`, `pyo3` (Python bindings), and `wasm` (WebAssembly bindings).
- `no_std` support: the crate is `no_std` with no features; `alloc` enables the
  heap-backed types without `std`.

### Notes

- Borsh and serde deserialization are bounded: a hostile length prefix cannot
  force a large preallocation before the payload is read.
- Python size-taking methods raise `MemoryError` rather than aborting the
  interpreter on absurd allocations; the Python UTF-8 classes index in Unicode
  characters, with `byte_len()` and the `Buf`-style methods working in bytes.
- WebAssembly `Buffer` write methods throw a catchable error instead of trapping
  the instance when full.
- Minimum supported Rust version is 1.85; the `bytes` dependency floor is 1.10.

[Unreleased]: https://github.com/al8n/smol-bytes/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/al8n/smol-bytes/releases/tag/v0.1.0
