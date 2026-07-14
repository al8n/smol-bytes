# smol-bytes

`smol-bytes` provides Rust byte buffers with inline storage for small values.
For ordinary copying and static constructors, values up to 62 bytes start
inline. Imported or owner-backed values, and retained shared heap views, can
remain heap-backed even when their current length is 62 bytes or less. Larger
or growable values can use `bytes`-backed heap storage. The crate also provides
mutable buffers and wrappers that guarantee valid UTF-8.

There are currently no published releases on crates.io, PyPI, or npm. All
packages in this repository build from source.

## Rust

Requires Rust 1.85 or newer. Add the crate as a pinned Git dependency:

```toml
[dependencies]
smol-bytes = { git = "https://github.com/al8n/smol-bytes", rev = "5956b34d60eee396da7ba5faee58e740b9608c8a" }
```

### Quick start

The root `Bytes` type is an alias for `shared::Bytes`. For ordinary copying and
static constructors, values up to 62 bytes start inline; imported or
owner-backed values and retained shared heap views may remain heap-backed below
that threshold.

```rust
use smol_bytes::Bytes;

let small = Bytes::from_static(b"identifier");
assert!(small.is_inline());

let cloned = small.clone();
assert_eq!(small, cloned);

let heap = Bytes::copy_from_slice(&[0_u8; 63]);
assert!(heap.is_heap());
assert_eq!(heap.len(), 63);
```

### Storage strategies

`shared::Bytes` preserves retained heap-backed views when operations shrink a
value. `compact::Bytes` may copy a shrinking view of 62 bytes or fewer into
inline storage. Ordinary copying and static constructors in either strategy
start values of up to 62 bytes inline.

```rust
use smol_bytes::{compact, shared, Buf};

let mut shared = shared::Bytes::from(vec![0_u8; 100]);
let mut compact = compact::Bytes::from(vec![0_u8; 100]);

shared.advance(70);
compact.advance(70);

assert_eq!(shared.len(), 30);
assert_eq!(compact.len(), 30);
assert!(shared.is_heap());
assert!(compact.is_inline());
```

### Types

| Type | Storage | Mutable | Purpose |
| --- | --- | :---: | --- |
| `Buffer` | Fixed inline bytes, up to 62 bytes | Yes | `no_std` fixed buffer |
| `Bytes` / `shared::Bytes` | Inline or shared heap-backed bytes | No | Immutable shared view |
| `compact::Bytes` | Inline or compacting heap-backed bytes | No | Immutable compacting view |
| `BytesMut` | Inline, then growable `bytes::BytesMut` storage | Yes | Mutable byte buffer |
| `Utf8Buffer` | Fixed inline, valid UTF-8 bytes | Yes | Small mutable UTF-8 value |
| `Utf8Bytes` / `compact::Utf8Bytes` | Shared or compacting UTF-8 bytes | No | Immutable UTF-8 value |
| `Utf8BytesMut` | Inline or growable valid UTF-8 bytes | Yes | Mutable UTF-8 value |

`BytesMut::split_to` and `BytesMut::split_off` return `Ok(BytesMut)` when the
output is growable heap storage and `Err(Buffer)` when the output is fixed
inline storage. The `try_split_*` variants add an outer bounds `Result`, so
their shape is `Result<Result<BytesMut, Buffer>, OutOfBounds>`.

Rust UTF-8 split and slice indices are byte offsets, not character counts, and
must be character boundaries. The fallible alternatives are `try_split_to`,
`try_split_off`, and `try_slice`.

## Features

| Feature | Description |
| --- | --- |
| `std` (default) | Standard-library support and the normal heap-backed types |
| `alloc` | Heap-backed types without `std` |
| `serde` | Serde support |
| `borsh` | Borsh support |
| `arbitrary` | `arbitrary` support for generated values |
| `quickcheck` | QuickCheck support |
| `pyo3` | Python bindings |
| `wasm` | WebAssembly bindings; implies `std` |

With no features, the crate is `no_std` and provides the fixed `Buffer` and
`Utf8Buffer` types. `alloc` adds heap-backed types without requiring `std`.

## Python from source

Python 3.11+, Rust, and `maturin` are required. From the repository root:

```bash
python -m venv .venv
source .venv/bin/activate
pip install maturin pytest
maturin develop --features pyo3 --manifest-path crates/smol-bytes/Cargo.toml
pytest tests/python -v
```

The root `smol_bytes` module contains `Buffer`, `BytesMut`, `Utf8Buffer`,
`Utf8Bytes`, and `Utf8BytesMut`. The `smol_bytes.shared` and
`smol_bytes.compact` modules expose immutable `Bytes` and `Utf8Bytes` types for
the shared and compact strategies.

```python
from smol_bytes import Utf8Bytes
from smol_bytes.shared import Bytes

raw = Bytes.from_bytes(b"abc")
assert raw.is_inline()
assert bytes(raw) == b"abc"

text = Utf8Bytes.from_str("café")
assert len(text) == 5
assert str(text) == "café"
```

## JavaScript, TypeScript, and WebAssembly from source

Use Node.js 20, `wasm-pack`, and the `wasm32-unknown-unknown` Rust target:

```bash
rustup target add wasm32-unknown-unknown
cd js
npm ci
npm run build
npm test
```

The root export contains the core, mutable, and shared UTF-8 types. The
`smol-bytes/shared` and `smol-bytes/compact` exports provide the corresponding
`Bytes` and `Utf8Bytes` strategy types. Byte conversions at the Wasm boundary
return copies.

```typescript
import { Utf8Bytes } from "smol-bytes";
import { Bytes as CompactBytes } from "smol-bytes/compact";

const raw = CompactBytes.fromBytes(new Uint8Array([1, 2, 3]));
console.assert(raw.isInline());
console.assert(raw.toBytes()[2] === 3);

const text = Utf8Bytes.fromString("café");
console.assert(text.len() === 5);
console.assert(text.toString() === "café");
```

## Performance characteristics

These are structural properties of the representations:

- Supported inline constructors avoid a separate backing allocation for inline values.
- The inline-capable handle is 64 bytes.
- Cloning an inline value copies its fixed value.
- Cloning an immutable non-inline `Bytes` or `Utf8Bytes` copies the underlying
  `bytes::Bytes` handle, usually incrementing a reference count; static backing
  does not. Cloning `BytesMut` or `Utf8BytesMut` copies contents.
- A shared heap-backed outbound conversion to `bytes::Bytes` may reuse its backing.
- Compact storage may copy up to 62 bytes when inlining a shrinking view.

Run the benchmark suite locally when investigating a change:

```bash
cargo bench
cargo bench --bench clone
cargo bench --bench split_to
```

## Development

The following commands match the main CI checks:

The final command requires `cargo-hack`; install it with `cargo install cargo-hack`.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-features --all-targets
cargo test --workspace --no-default-features --features std,serde,borsh,arbitrary,quickcheck
cargo rustc --package smol-bytes --lib --no-default-features --crate-type rlib
cargo test --package smol-bytes --no-default-features --features alloc,quickcheck
cargo hack --workspace --feature-powerset --include-features alloc,serde,borsh,arbitrary rustc --lib --crate-type rlib
```

## Contributing and links

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidance. Build and
open the local API documentation with:

```bash
cargo doc --package smol-bytes --open
```

- Repository: <https://github.com/al8n/smol-bytes>
- Issues: <https://github.com/al8n/smol-bytes/issues>

## License

Licensed under either MIT OR Apache-2.0:

- [Apache License 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)
