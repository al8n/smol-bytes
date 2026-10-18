# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-10-18

See the [0.2 migration guide](MIGRATING-0.2.md) for side-by-side Rust, Python,
and JavaScript migration examples.

### Breaking

- **Rust:** `BytesMut::split`, `split_to`, and `split_off` now return
  `BytesMut` directly; `try_split_to` and `try_split_off` return
  `Result<BytesMut, OutOfBounds>`, `try_split` is removed, and total
  `unsplit` consumes its argument and returns `()`.
- **Rust:** `Utf8Buffer` no longer implements `From<&str>`; use `TryFrom`,
  `try_from_str`, or `FromStr` to handle its fixed 62-byte capacity.
- **Rust:** `Buffer::resize` and `try_resize` require an explicit fill byte,
  and `Utf8Error` is replaced by `Utf8IndexError` with boundary and full-range
  context.
- **Rust:** `smol_bytes::TryGetError` is now a crate-owned fixed-width
  inherent `Buffer` underflow error, not the old upstream `bytes` alias. Code
  that handles `bytes::TryGetError` should use `smol_bytes::BufTryGetError`
  when `alloc` or `std` is enabled; variable-width inherent readers return
  `TryGetIntegerError`. `Utf8Buf` and `Utf8BufMut` are sealed.
- **Python:** raw classes rename `from_str`/`to_string` to
  `from_utf8`/`decode_utf8`; `str(raw)` is now a total byte representation.
  Ambiguous UTF-8 byte/character methods are replaced by explicit
  `*_chars` and `*_bytes` names. In particular, removed
  `Utf8Buffer.len()` (the 0.1 byte count) becomes `byte_len()`; built-in
  `len(value)` is the Unicode scalar count.
- **JavaScript/Wasm:** raw classes rename `fromString`/raw `toString` to
  `fromUtf8`/`decodeUtf8`; `fromStatic` is removed from both raw and UTF-8
  public classes, while UTF-8 construction uses `fromString`. UTF-8 byte
  operations use explicit `byteLength`, `charLength`, and `*Bytes` names.

### Added

- **Rust:** `FromUtf8Error<B>` preserves owned invalid input for recovery;
  owned UTF-8 wrappers expose `from_utf8`. `TryGetError` and
  `TryGetIntegerError` provide crate-owned inherent-reader errors.
- **Rust:** safe mutable `str` access for `Utf8Buffer` and `Utf8BytesMut`,
  fallible UTF-8 mutation through `Utf8BufMut::CapacityError`, and
  `fmt::Write` for `Utf8Buffer`.
- **Rust:** immutable `Utf8Bytes<S>` view operations, uniqueness and
  recoverable mutable conversion; root-default freeze methods; strategy-aware
  consuming conversions; `Cow`/chunk collection support; and `OsStr`/`Path`
  views under `std`.
- **Rust:** cross-strategy raw/UTF-8 comparisons and conversions, plus byte
  JSON schemas for fixed/mutable/immutable raw buffers. `Buffer` bounds array
  input with `maxItems` and string input with `maxLength`; the latter is a
  scalar-count bound and byte capacity remains enforced during deserialization.
  The `std` feature now implies `alloc`; `bytes` remains a compatibility alias
  and `quickcheck` implies `std`.
- **Python:** canonical root shared `Bytes` and `Utf8Bytes`, with exact shared
  aliases that preserve historic shared-module pickle loading; `py.typed` and
  complete stubs; bytes-like input support; independent mutable slices;
  destructive `freeze_shared`/`freeze_compact`; and immutable `to_mutable`.
- **Python:** explicit Unicode-scalar and byte-unit APIs, normalized exception
  taxonomy, and non-consuming snapshot `unsplit` (self-alias raises
  `RuntimeError` atomically).
- **JavaScript/Wasm:** canonical root shared immutable classes and exact shared
  aliases, distinct compact classes, `clone`, mutable `slice`, destructive
  freeze methods, immutable `toMutable`, snapshot `unsplit`, raw
  variable-width integer operations, and additional Buffer/BytesMut parity.
- **JavaScript/Wasm:** strict number/BigInt and malformed UTF-16 validation,
  iterable and disposable values/iterators, TypeScript 5.2 declarations, and
  packed-package verification across all entry points.

### Changed

- **Python:** raw constructors and mutable writes accept bytes, bytearray, and
  readable contiguous byte buffers; UTF-8 classes retain string-oriented APIs
  while indexing and iteration consistently use Unicode scalar values.
- **JavaScript/Wasm:** root, shared, and compact entry points independently
  install validation, iteration, and disposal behavior. Numeric and boundary
  failures are checked before Wasm coercion where deterministic.

## [0.1.8] - 2026-10-13

### Added

- Iterator collection and extension support for `Utf8Bytes` and `Utf8BytesMut`.

### Security

- PyPI and npm releases use GitHub Actions OIDC trusted publishing from their
  protected environments.

## [0.1.7] - 2026-10-13

### Added

- `format_utf8bytes!` and `format_compact_utf8bytes!` macros, `ToUtf8Bytes`
  formatting conversions, and `StrExt` case-conversion and replacement helpers
  for both immutable strategies.

### Changed

- Replaced the unmaintained `paste` package with maintained `pastey` through a
  Cargo package alias, preserving `paste::paste!` call sites and the public API.

### Security

- Crate releases use GitHub Actions OIDC trusted publishing from the protected
  `crates-io` environment.

## [0.1.6] - 2026-09-30

### Added

- Add `core::fmt::Write` implementation for `Utf8BytesMut`

## [0.1.5] - 2026-09-14

### Added

- `FromStr` support for `shared::Utf8Bytes`, `compact::Utf8Bytes`, and
  `Utf8BytesMut` with `Infallible`, plus fallible `Utf8Buffer` parsing with
  its existing `TryPutError` capacity error.
- Optional `schemars` support for all UTF-8 wrappers. Their schemas are the
  ordinary JSON string schema regardless of storage strategy.

### Changed

- The documented and CI-tested library MSRV is Rust 1.95, matching the
  workspace manifest.

## [0.1.4] - 2026-08-22

### Changed

- `Utf8Bytes::new`, `Utf8Bytes::from_static` and `Utf8BytesMut::new` are now
  `const fn`, so a UTF-8 constant no longer has to be built at run time. The
  storage they wrap was already const-constructible — `RawBytes::new`,
  `RawBytes::from_static` and `BytesMut::new` — and both branches of
  `from_static` hold in a constant: a string within `INLINE_CAP` is copied into
  the inline buffer, a longer one borrows the static allocation. The remaining
  constructors stay non-`const` because they allocate
  (`BytesMut::with_capacity`, `Utf8BytesMut::with_capacity`, `BytesMut::zeroed`,
  `RawBytes::copy_from_slice`, `shared::Bytes::from_owner`) or reach a trait
  method that is not const on stable (`Utf8Buffer::try_from_str`, via `TryFrom`
  and `?`). This is additive; no signature changed and the MSRV stays at 1.85.

## [0.1.3] - 2026-08-01

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
- Optional `sqlx` feature implementing `Type`, `Encode` and `Decode` for
  `shared::Bytes`, `compact::Bytes`, `shared::Utf8Bytes` and
  `compact::Utf8Bytes`. The impls are generic over `Database`, delegating to
  `[u8]`/`&[u8]` and `str`/`&str`, so PostgreSQL, MySQL and SQLite are all
  covered. Both directions borrow: a decoded value of at most `INLINE_CAP`
  bytes is built inline with no heap allocation at all, and a longer one costs
  a single copy into a single allocation. UTF-8 is validated on the decode
  path by the driver rather than assumed. Decoding through `&[u8]` carries one
  restriction — PostgreSQL will not lend `BYTEA` out in a simple (unprepared)
  query, so code on that path decodes the byte types as `Vec<u8>` and
  converts. `Buffer` and `Utf8Buffer` are deliberately excluded:
  they are capped at `INLINE_CAP` and a database column is of unbounded
  length, so binding them would fail at runtime on data the program did not
  choose. The feature implies `std` and requires Rust 1.94.

### Fixed

- `LICENSE-APACHE` and `LICENSE-MIT` are now included in the published crate.
  They live at the workspace root, outside the package directory, so the
  README's links to them resolved to nothing on crates.io and docs.rs.

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

[Unreleased]: https://github.com/al8n/smol-bytes/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/al8n/smol-bytes/compare/v0.1.8...v0.2.0
[0.1.8]: https://github.com/al8n/smol-bytes/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/al8n/smol-bytes/compare/a56cfeca004c750ef4df7b42af05a9c8d6b981ba...v0.1.7
[0.1.6]: https://github.com/al8n/smol-bytes/compare/8d88fde61726dd195d0aee3f707a4862bf9097e3...a56cfeca004c750ef4df7b42af05a9c8d6b981ba
[0.1.5]: https://github.com/al8n/smol-bytes/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/al8n/smol-bytes/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/al8n/smol-bytes/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/al8n/smol-bytes/compare/v0.1.1...v0.1.2
[0.1.0]: https://github.com/al8n/smol-bytes/releases/tag/v0.1.0
