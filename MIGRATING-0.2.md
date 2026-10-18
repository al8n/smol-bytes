# Migrating to smol-bytes 0.2

smol-bytes 0.2 makes fallibility, ownership, and UTF-8 units explicit across
Rust, Python, and JavaScript/Wasm. The inline storage threshold is unchanged;
the migration is primarily an API migration.

## Rust

### Core APIs

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

```rust
use smol_bytes::{BytesMut, Utf8Buffer};

let mut text = Utf8Buffer::try_from("header")?;
text.try_push_str(": value")?;

let mut bytes = BytesMut::from(&b"abcdef"[..]);
let head = bytes.try_split_to(3)?;
assert_eq!(&head[..], b"abc");
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Strategies, UTF-8 views, and features

The root aliases use the shared strategy. Compact values remain explicit and
distinct:

```rust
use smol_bytes::{compact, Utf8Bytes, Utf8BytesMut};

let shared: Utf8Bytes = Utf8BytesMut::from("payload").freeze();
let compact: compact::Utf8Bytes = shared.into();
```

Use `freeze()` for the root shared result, or `freeze_shared()` and
`freeze_compact()` when the target strategy matters. Both
`Utf8Bytes` (shared) and `compact::Utf8Bytes` support byte-indexed,
boundary-checked view operations such as `clear`, `truncate`, `advance`,
`is_unique`, and recoverable `try_into_mut`.

`Utf8Buffer` and `Utf8BytesMut` now expose safe mutable string access through
`as_mut_str`, `AsMut<str>`, `BorrowMut<str>`, `DerefMut<Target = str>`, and
mutable string-range indexing. `Utf8Buf` and `Utf8BufMut` are sealed; generic
code should use their fallible methods and associated `CapacityError`.

Feature changes: `std` implies `alloc`; `bytes` remains a compatibility alias
for `alloc`; and `quickcheck` implies `std`. A no-feature build still offers
the fixed `Buffer` and `Utf8Buffer`; use `alloc` for heap-backed types without
`std`.

## Python

### Imports, strategies, and typing

The root module now exports canonical shared immutable classes:

```python
from smol_bytes import Bytes, BytesMut, Utf8Bytes, Utf8BytesMut
from smol_bytes.compact import Bytes as CompactBytes
```

`smol_bytes.shared.Bytes` and `smol_bytes.shared.Utf8Bytes` are exact aliases
of root classes. Historic shared-module pickle references continue to load.
Compact classes remain distinct. The distribution includes `py.typed` and
final-surface stubs.

### Raw bytes and ownership

| Before 0.2 | 0.2 replacement |
| --- | --- |
| `Raw.from_str(text)` | `Raw.from_utf8(text)` |
| `raw.to_string()` | `raw.decode_utf8()` |
| `str(raw)` as a decoder | `str(raw)` is total and equals `str(bytes(raw))`; use `decode_utf8()` to decode |
| `Utf8Buffer.len()` byte count | `value.byte_len()` | Built-in `len(value)` is now the Unicode scalar count. |
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

### Explicit UTF-8 units and errors

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
failure, `UnicodeEncodeError` for malformed Python surrogates, and `MemoryError`
for allocation failure.

## JavaScript and WebAssembly

### Entry points, names, and ownership

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

### Explicit UTF-8 units, validation, and TypeScript

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

## Upgrade checklist

1. Update dependency requirements to 0.2.
2. Apply the table for each binding surface you use.
3. Search for removed raw-string and ambiguous UTF-8 unit names.
4. Add ownership tests around `freeze`, `to_mutable`, and `unsplit`.
5. Run the feature or binding validation commands relevant to your deployment.
