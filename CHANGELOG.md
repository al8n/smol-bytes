# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-10-18

The complete Rust, Python, and JavaScript migration guide is included below.

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
  owned UTF-8 wrappers expose `from_utf8`. `Utf8BytesMut`, `Utf8Bytes`, and
  `compact::Utf8Bytes` also expose `from_utf8_lossy`, preserving the owned
  representation for valid input. `TryGetError` and `TryGetIntegerError`
  provide crate-owned inherent-reader errors.
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
- **Rust:** `RawBytes<S>` and `Utf8Bytes<S>` now have reference-checked
  `slice_ref`/`try_slice_ref` view creation; non-empty references must belong
  to the current visible view, while empty references produce a canonical empty
  value. `Buffer::try_reclaim` is available without `alloc`, with inherent
  UTF-8 forwarding methods. Fixed `Utf8Buffer` values convert directly into
  mutable or either immutable UTF-8 strategy, and all UTF-8 storage types have
  scalar-value `pop`.
- **Rust:** `BytesMut::try_make_inline` and
  `Utf8BytesMut::try_make_inline` explicitly compact a current short heap view
  into inline storage without allocating.
- **Rust:** `Utf8Buffer` and `Utf8BytesMut` now offer `insert_str`/
  `try_insert_str`, `replace_range`/`try_replace_range`, and `retain`; fixed
  edit failures use `Utf8EditError`. Their `drain`/`try_drain` methods return
  `Utf8Drain`, whose drop removes its full selected range.
- **Rust:** raw `Buffer` and `BytesMut` support fallible byte-range
  replacement and self-extension; all four mutable buffer types support
  `extend_from_within`. UTF-8 mutable buffers add scalar `insert`/`remove`,
  fixed `Utf8Buffer::try_write_fmt`, and recoverable `try_into_buffer`
  conversions from raw and validated UTF-8 heap views. Checked variable-width
  integer writers reject invalid widths and unrepresentable values without
  truncating, using `TryPutIntegerCheckedError`.
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
- **Python/JavaScript/Wasm:** UTF-8 wrappers add `pop`, scalar/string
  insertion, range replacement, lossy constructors, and explicit mutable
  inline compaction. Python indices are Unicode-scalar indices; JavaScript/
  Wasm mutation indices are UTF-8 byte offsets and must be character
  boundaries.

### Changed

- **Python:** raw constructors and mutable writes accept bytes, bytearray, and
  readable contiguous byte buffers; UTF-8 classes retain string-oriented APIs
  while indexing and iteration consistently use Unicode scalar values.
- **Python:** PyO3 integration targets 0.30; the packaged extension retains
  Python >= 3.11 and abi3-py311.
- **JavaScript/Wasm:** root, shared, and compact entry points independently
  install validation, iteration, and disposal behavior. Numeric and boundary
  failures are checked before Wasm coercion where deterministic.

### Migration guide

smol-bytes 0.2 makes fallibility, ownership, and UTF-8 units explicit across
Rust, Python, and JavaScript/Wasm. The inline storage threshold is unchanged;
the migration is primarily an API migration.

#### Rust

##### Core APIs

| Before 0.2 | 0.2 replacement | Why |
| --- | --- | --- |
| `Utf8Buffer::from(text)` | `Utf8Buffer::try_from(text)?` or `Utf8Buffer::try_from_str(text)?` | Fixed-capacity construction can fail with `TryPutError`. `FromStr` remains available. |
| `BytesMut::try_split()` | `BytesMut::split()` | `split()` now always returns `BytesMut`; the `try_split` alias is removed. |
| fallible `split_to` / `split_off` wrappers | `try_split_to(at)` / `try_split_off(at)` | These return `Result<BytesMut, OutOfBounds>` and are atomic on bounds failure. |
| `Buffer::resize(new_len)` | `resize(new_len, fill)` | `try_resize(new_len, fill)` is atomic when it reports an error. |
| `Utf8Error` | `Utf8IndexError` | The new error preserves boundary, index, and complete range context. |
| owned UTF-8 conversion error | `FromUtf8Error<B>` | Call `utf8_error()` and `into_bytes()` to inspect and recover the original owner. |
| `smol_bytes::TryGetError` used as `bytes::TryGetError` | `smol_bytes::BufTryGetError` | Use this upstream alias for `bytes::Buf` readers when `alloc` or `std` is enabled. |
| fixed-width inherent `Buffer::try_get_*` underflow | new crate-owned `smol_bytes::TryGetError` | This is no longer the upstream bytes alias. |
| inherent variable-width reader error | `TryGetIntegerError` | `try_get_uint{,_le,_ne}` and `try_get_int{,_le,_ne}` distinguish invalid widths from missing bytes. |
| derive a view from a borrowed subslice | `slice_ref(subset)` or `try_slice_ref(subset)` | Non-empty subsets must be address-contained in the current visible view; empty subsets return a canonical empty value. |
| reuse a consumed fixed-buffer prefix | `try_reclaim(additional)` | Available on `Buffer` without `alloc`, and forwarded by `Utf8Buffer` and `Utf8BytesMut`; it never allocates. |
| remove the final UTF-8 scalar manually | `pop()` | Available on `Utf8Buffer`, `Utf8BytesMut`, and both `Utf8Bytes` strategies; it removes one Unicode scalar, not a grapheme. |
| turn a fixed UTF-8 buffer immutable or growable | `Utf8BytesMut::from(buffer)`, `Utf8Bytes::from(buffer)`, or `compact::Utf8Bytes::from(buffer)` | These move the validated inline storage without UTF-8 revalidation or heap allocation. |
| manually shift bytes to replace or append a visible range | `try_replace_range` or `try_extend_from_within` | Raw methods use byte offsets; fixed buffers validate capacity before mutation. |
| manually encode or decode one UTF-8 scalar at an offset | `try_insert(index, ch)` or `try_remove(index)` | UTF-8 indices are byte offsets and must be character boundaries. |
| rely on variable-width integer truncation | `try_put_{u,int}_checked{,_le,_ne}` | Checked writers reject values outside the exact signed or unsigned width. |
| copy a short heap view into a fixed buffer manually | `try_into_buffer()` | A value over 62 visible bytes is returned unchanged; inline source storage moves with its cursor preserved. |

```rust
use smol_bytes::{BytesMut, Utf8Buffer};

let mut text = Utf8Buffer::try_from("header")?;
text.try_push_str(": value")?;

let mut bytes = BytesMut::from(&b"abcdef"[..]);
let head = bytes.try_split_to(3)?;
assert_eq!(&head[..], b"abc");
# Ok::<(), Box<dyn std::error::Error>>(())
```

##### New view, reclamation, and scalar helpers

```rust
use smol_bytes::{compact, Buffer, Utf8Buffer, Utf8Bytes, Utf8BytesMut};

let source = Utf8Bytes::from("  café  ");
let view = source.slice_ref(source.as_str().trim());
assert_eq!(view.as_str(), "café");

let mut buffer = Buffer::from(*b"abcd");
buffer.advance(2);
assert!(!buffer.try_reclaim(usize::MAX));
assert!(buffer.try_reclaim(60));

let fixed = Utf8Buffer::try_from("aé").unwrap();
let mut mutable: Utf8BytesMut = fixed.into();
assert_eq!(mutable.pop(), Some('é'));

let shared: Utf8Bytes = Utf8Buffer::try_from("café").unwrap().into();
let compact: compact::Utf8Bytes = Utf8Buffer::try_from("café").unwrap().into();
assert_eq!(shared.as_str(), compact.as_str());
```

##### In-place edits, strict fields, and formatting

```rust
use smol_bytes::{Buffer, Utf8Buffer};

let mut bytes = Buffer::from(*b"abcd");
bytes.try_replace_range(1..3, b"XY").unwrap();
bytes.try_extend_from_within(1..3).unwrap();
assert_eq!(bytes.as_slice(), b"aXYdXY");

let mut field = Buffer::new();
field.try_put_uint_checked(0xff, 1).unwrap();
assert!(field.try_put_uint_checked(0x100, 1).is_err());

let mut text = Utf8Buffer::try_from("aé").unwrap();
text.try_insert(1, '€').unwrap();
assert_eq!(text.try_remove(1).unwrap(), '€');
```

##### Strategies, UTF-8 views, and features

The root aliases use the shared strategy. Compact values remain explicit and
distinct:

```rust
use smol_bytes::{compact, Utf8Bytes, Utf8BytesMut};

let shared: Utf8Bytes = Utf8BytesMut::from("payload").freeze();
let compact: compact::Utf8Bytes = shared.into();
```

Use `freeze()` for the root shared result, or `freeze_shared()` and
`freeze_compact()` when the target strategy matters. Both `Utf8Bytes`
(shared) and `compact::Utf8Bytes` support byte-indexed, boundary-checked view
operations such as `clear`, `truncate`, `advance`, `is_unique`, and
recoverable `try_into_mut`.

`Utf8Buffer` and `Utf8BytesMut` now expose safe mutable string access through
`as_mut_str`, `AsMut<str>`, `BorrowMut<str>`, `DerefMut<Target = str>`, and
mutable string-range indexing. `Utf8Buf` and `Utf8BufMut` are sealed; generic
code should use their fallible methods and associated `CapacityError`.

Feature changes: `std` implies `alloc`; `bytes` remains a compatibility alias
for `alloc`; and `quickcheck` implies `std`. A no-feature build still offers
the fixed `Buffer` and `Utf8Buffer`; use `alloc` for heap-backed types without
`std`.

#### Python

##### Imports, strategies, and typing

The root module now exports canonical shared immutable classes:

```python
from smol_bytes import Bytes, BytesMut, Utf8Bytes, Utf8BytesMut
from smol_bytes.compact import Bytes as CompactBytes
```

`smol_bytes.shared.Bytes` and `smol_bytes.shared.Utf8Bytes` are exact aliases
of root classes. Historic shared-module pickle references continue to load.
Compact classes remain distinct. The distribution includes `py.typed` and
public API type stubs.

##### Raw bytes and ownership

| Before 0.2 | 0.2 replacement |
| --- | --- |
| `Raw.from_str(text)` | `Raw.from_utf8(text)` |
| `raw.to_string()` | `raw.decode_utf8()` |
| `str(raw)` as a decoder | `str(raw)` is total and equals `str(bytes(raw))`; use `decode_utf8()` to decode |
| `Utf8Buffer.len()` byte count | `value.byte_len()`; built-in `len(value)` is now the Unicode scalar count |
| manual mutable byte copy | `BytesMut.slice(start, end)` |
| manual immutable-to-mutable copy | `to_mutable()` |
| implicit freeze ownership | `freeze_shared()` or `freeze_compact()` drains the mutable source |

`from_bytes()` and mutable `put_slice()` accept `bytes`, `bytearray`, and a
readable one-dimensional C-contiguous byte-format `memoryview`. Input is
snapshotted before mutation.

```python
raw = Bytes.from_utf8("hello")
mutable = BytesMut.from_bytes(bytearray(b"hello"))
part = mutable.slice(1, 4)
frozen = mutable.freeze_shared()
assert bytes(mutable) == b""
assert bytes(part) == b"ell"
assert bytes(frozen.to_mutable()) == b"hello"
```

Python `unsplit()` intentionally differs from Rust: it snapshots and appends
`other`, keeps `other` unchanged, returns `None`, and rejects self-aliasing
with an atomic `RuntimeError`.

##### Explicit UTF-8 units and errors

Python UTF-8 classes use Unicode scalar values for `len`, indexing, slicing,
and iteration. Named methods now state their unit:

| Intent | 0.2 API |
| --- | --- |
| scalar count | `char_len()` |
| encoded byte count | `byte_len()` |
| character split/slice | `split_to_chars()`, `split_off_chars()`, `slice_chars()` |
| fixed/mutable character truncation | `truncate_chars()` |
| encoded byte cursor | `remaining_bytes()`, `advance_bytes()` |

The ambiguous UTF-8 `split_to`, `split_off`, `slice`, `truncate`, `remaining`,
and `advance` names are removed. Numeric UTF-8 getters remain byte-oriented.

The normalized public errors are `TypeError` for wrong types, `ValueError` for
invalid numeric values/ranges or scalar boundaries, `IndexError` for offsets,
`BufferError` for insufficient data/capacity, `UnicodeDecodeError` for decode
failure, `UnicodeEncodeError` for malformed Python surrogates, and
`MemoryError` for allocation failure.

#### JavaScript and WebAssembly

##### Entry points, names, and ownership

The root package exports shared `Bytes` and `Utf8Bytes`. `smol-bytes/shared`
exports those same class objects; `smol-bytes/compact` exports distinct compact
classes.

| Before 0.2 | 0.2 replacement |
| --- | --- |
| `Buffer.fromString(text)` | `Buffer.fromUtf8(text)` |
| raw `value.toString()` | `value.decodeUtf8()` |
| raw or UTF-8 `fromStatic` | removed; UTF-8 construction uses `fromString()` |
| implicit clone/copy choice | `clone()` on each public byte/string value class |
| manual mutable copy | immutable `toMutable()` |
| consuming mutable merge | snapshot `unsplit(other)`; `other` stays valid and self-alias doubles contents |

`BytesMut.slice(start, end)` and `Utf8BytesMut.sliceBytes(start, end)` return
independent mutable copies. `freezeShared()` and `freezeCompact()` drain their
mutable source into the selected strategy.

##### Explicit UTF-8 units, validation, and TypeScript

UTF-8 classes use `byteLength()`, `charLength()`, `remainingBytes()`,
`advanceBytes()`, `splitToBytes()`, `splitOffBytes()`, `sliceBytes()`, and
`truncateBytes()` where applicable. Byte offsets must be UTF-8 boundaries.

Numbers and BigInts are validated before Wasm coercion: invalid type is
`TypeError`, while non-finite, fractional, unsafe, out-of-range, or
width-unrepresentable values are `RangeError`. String constructors and
mutators reject isolated UTF-16 surrogates with `TypeError`.

```ts
import { Bytes, BytesMut, Utf8Bytes } from "smol-bytes";
import { Bytes as CompactBytes } from "smol-bytes/compact";

const shared = Bytes.fromUtf8("hello");
const compact = CompactBytes.fromUtf8("hello");
const frozen = BytesMut.fromUtf8("ready").freezeShared();

const text = Utf8Bytes.fromString("café");
console.assert(text.byteLength() === 5);
console.assert(text.charLength() === 4);
```

Public values and iterators implement `[Symbol.iterator]()` and
`[Symbol.dispose]()` while retaining `free()`. TypeScript consumers need
TypeScript 5.2 or later for disposable declarations. The packed package is
verified in strict TypeScript mode without `skipLibCheck`.

#### Upgrade checklist

1. Update dependency requirements to 0.2.
2. Apply the table for each binding surface you use.
3. Search for removed raw-string and ambiguous UTF-8 unit names.
4. Add ownership tests around `freeze`, `to_mutable`, and `unsplit`.
5. Run the feature or binding validation commands relevant to your deployment.

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
