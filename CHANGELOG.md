# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Optional `async-graphql` feature giving `shared::Bytes` and `compact::Bytes`
  the GraphQL scalar `Bytes`, backed by `Value::Binary`, and
  `shared::Utf8Bytes` and `compact::Utf8Bytes` the built-in scalar `String`,
  backed by `Value::String`. A scalar name is a wire type, and the two
  strategies behind each are a storage decision a schema cannot act on, so
  they share one name; `parse` and `to_value` still resolve on the Rust type.
  `Bytes` coexists with `async-graphql`'s own `bytes::Bytes` scalar, and no
  `scalar String` is ever declared — redeclaring a built-in would be invalid
  SDL. `shared::Bytes` parses and serialises without copying. `Buffer` and
  `Utf8Buffer` are deliberately excluded: they are capped at `INLINE_CAP` and
  a GraphQL input is of unbounded length, so a scalar for them would fail at
  runtime on data the program did not choose. The feature implies `std` and
  requires Rust 1.89.

## [0.1.2] - 2026-07-18

### Fixed

- `smol-bytes` is now built as a plain `rlib` instead of
  `crate-type = ["cdylib", "rlib"]`, so it can be used as a dependency in
  `no_std` targets. The forced `cdylib` previously pulled in a `#[panic_handler]`
  requirement that broke every downstream `no_std` build.

### Changed

- The Python (`pyo3`) and WebAssembly (`wasm`) bindings now ship through two thin
  wrapper crates, `smol-bytes-py` and `smol-bytes-wasm`, which provide the
  `cdylib` artifacts. All binding code stays in `smol-bytes` (feature-gated);
  maturin and wasm-pack now target the wrapper crates.

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

[Unreleased]: https://github.com/al8n/smol-bytes/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/al8n/smol-bytes/compare/v0.1.1...v0.1.2
[0.1.0]: https://github.com/al8n/smol-bytes/releases/tag/v0.1.0
