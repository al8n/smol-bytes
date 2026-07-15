<div align="center">
<h1>smol-bytes</h1>
</div>
<div align="center">

Small, clone-efficient byte buffer types.

[<img alt="github" src="https://img.shields.io/badge/github-al8n/smol--bytes-8da0cb?style=for-the-badge&logo=Github" height="22">][Github-url]
<img alt="LoC" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fgist.githubusercontent.com%2Fal8n%2F327b2a8aef9003246e45c6e47fe63937%2Fraw%2Fsmol-bytes" height="22">
[<img alt="Build" src="https://img.shields.io/github/actions/workflow/status/al8n/smol-bytes/ci.yml?logo=Github-Actions&style=for-the-badge" height="22">][CI-url]
[<img alt="codecov" src="https://img.shields.io/codecov/c/gh/al8n/smol-bytes?style=for-the-badge&token=6R3QFWRWHL&logo=codecov" height="22">][codecov-url]

[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-smol--bytes-66c2a5?style=for-the-badge&labelColor=555555&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">][doc-url]
[<img alt="crates.io" src="https://img.shields.io/crates/v/smol-bytes?style=for-the-badge&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJMYXllcl8xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiB4PSIwcHgiIHk9IjBweCINCgkgdmlld0JveD0iMCAwIDUxMiA1MTIiIHhtbDpzcGFjZT0icHJlc2VydmUiPg0KPGc+DQoJPGc+DQoJCTxwYXRoIGQ9Ik0yNTYsMEwzMS41MjgsMTEyLjIzNnYyODcuNTI4TDI1Niw1MTJsMjI0LjQ3Mi0xMTIuMjM2VjExMi4yMzZMMjU2LDB6IE0yMzQuMjc3LDQ1Mi41NjRMNzQuOTc0LDM3Mi45MTNWMTYwLjgxDQoJCQlsMTU5LjMwMyw3OS42NTFWNDUyLjU2NHogTTEwMS44MjYsMTI1LjY2MkwyNTYsNDguNTc2bDE1NC4xNzQsNzcuMDg3TDI1NiwyMDIuNzQ5TDEwMS44MjYsMTI1LjY2MnogTTQzNy4wMjYsMzcyLjkxMw0KCQkJbC0xNTkuMzAzLDc5LjY1MVYyNDAuNDYxbDE1OS4zMDMtNzkuNjUxVjM3Mi45MTN6IiBmaWxsPSIjRkZGIi8+DQoJPC9nPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPC9zdmc+DQo=" height="22">][crates-url]
[<img alt="crates.io" src="https://img.shields.io/crates/d/smol-bytes?color=critical&logo=data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBzdGFuZGFsb25lPSJubyI/PjwhRE9DVFlQRSBzdmcgUFVCTElDICItLy9XM0MvL0RURCBTVkcgMS4xLy9FTiIgImh0dHA6Ly93d3cudzMub3JnL0dyYXBoaWNzL1NWRy8xLjEvRFREL3N2ZzExLmR0ZCI+PHN2ZyB0PSIxNjQ1MTE3MzMyOTU5IiBjbGFzcz0iaWNvbiIgdmlld0JveD0iMCAwIDEwMjQgMTAyNCIgdmVyc2lvbj0iMS4xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHAtaWQ9IjM0MjEiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkzIiB3aWR0aD0iNDgiIGhlaWdodD0iNDgiIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIj48ZGVmcz48c3R5bGUgdHlwZT0idGV4dC9jc3MiPjwvc3R5bGU+PC9kZWZzPjxwYXRoIGQ9Ik00NjkuMzEyIDU3MC4yNHYtMjU2aDg1LjM3NnYyNTZoMTI4TDUxMiA3NTYuMjg4IDM0MS4zMTIgNTcwLjI0aDEyOHpNMTAyNCA2NDAuMTI4QzEwMjQgNzgyLjkxMiA5MTkuODcyIDg5NiA3ODcuNjQ4IDg5NmgtNTEyQzEyMy45MDQgODk2IDAgNzYxLjYgMCA1OTcuNTA0IDAgNDUxLjk2OCA5NC42NTYgMzMxLjUyIDIyNi40MzIgMzAyLjk3NiAyODQuMTYgMTk1LjQ1NiAzOTEuODA4IDEyOCA1MTIgMTI4YzE1Mi4zMiAwIDI4Mi4xMTIgMTA4LjQxNiAzMjMuMzkyIDI2MS4xMkM5NDEuODg4IDQxMy40NCAxMDI0IDUxOS4wNCAxMDI0IDY0MC4xOTJ6IG0tMjU5LjItMjA1LjMxMmMtMjQuNDQ4LTEyOS4wMjQtMTI4Ljg5Ni0yMjIuNzItMjUyLjgtMjIyLjcyLTk3LjI4IDAtMTgzLjA0IDU3LjM0NC0yMjQuNjQgMTQ3LjQ1NmwtOS4yOCAyMC4yMjQtMjAuOTI4IDIuOTQ0Yy0xMDMuMzYgMTQuNC0xNzguMzY4IDEwNC4zMi0xNzguMzY4IDIxNC43MiAwIDExNy45NTIgODguODMyIDIxNC40IDE5Ni45MjggMjE0LjRoNTEyYzg4LjMyIDAgMTU3LjUwNC03NS4xMzYgMTU3LjUwNC0xNzEuNzEyIDAtODguMDY0LTY1LjkyLTE2NC45MjgtMTQ0Ljk2LTE3MS43NzZsLTI5LjUwNC0yLjU2LTUuODg4LTMwLjk3NnoiIGZpbGw9IiNmZmZmZmYiIHAtaWQ9IjM0MjIiIGRhdGEtc3BtLWFuY2hvci1pZD0iYTMxM3guNzc4MTA2OS4wLmkwIiBjbGFzcz0iIj48L3BhdGg+PC9zdmc+&style=for-the-badge" height="22">][crates-url]
<img alt="license" src="https://img.shields.io/badge/License-Apache%202.0/MIT-blue.svg?style=for-the-badge&fontColor=white&logoColor=f5c076&logo=data:image/svg+xml;base64,PCFET0NUWVBFIHN2ZyBQVUJMSUMgIi0vL1czQy8vRFREIFNWRyAxLjEvL0VOIiAiaHR0cDovL3d3dy53My5vcmcvR3JhcGhpY3MvU1ZHLzEuMS9EVEQvc3ZnMTEuZHRkIj4KDTwhLS0gVXBsb2FkZWQgdG86IFNWRyBSZXBvLCB3d3cuc3ZncmVwby5jb20sIFRyYW5zZm9ybWVkIGJ5OiBTVkcgUmVwbyBNaXhlciBUb29scyAtLT4KPHN2ZyBmaWxsPSIjZmZmZmZmIiBoZWlnaHQ9IjgwMHB4IiB3aWR0aD0iODAwcHgiIHZlcnNpb249IjEuMSIgaWQ9IkNhcGFfMSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIiB4bWxuczp4bGluaz0iaHR0cDovL3d3dy53My5vcmcvMTk5OS94bGluayIgdmlld0JveD0iMCAwIDI3Ni43MTUgMjc2LjcxNSIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSIgc3Ryb2tlPSIjZmZmZmZmIj4KDTxnIGlkPSJTVkdSZXBvX2JnQ2FycmllciIgc3Ryb2tlLXdpZHRoPSIwIi8+Cg08ZyBpZD0iU1ZHUmVwb190cmFjZXJDYXJyaWVyIiBzdHJva2UtbGluZWNhcD0icm91bmQiIHN0cm9rZS1saW5lam9pbj0icm91bmQiLz4KDTxnIGlkPSJTVkdSZXBvX2ljb25DYXJyaWVyIj4gPGc+IDxwYXRoIGQ9Ik0xMzguMzU3LDBDNjIuMDY2LDAsMCw2Mi4wNjYsMCwxMzguMzU3czYyLjA2NiwxMzguMzU3LDEzOC4zNTcsMTM4LjM1N3MxMzguMzU3LTYyLjA2NiwxMzguMzU3LTEzOC4zNTcgUzIxNC42NDgsMCwxMzguMzU3LDB6IE0xMzguMzU3LDI1OC43MTVDNzEuOTkyLDI1OC43MTUsMTgsMjA0LjcyMywxOCwxMzguMzU3UzcxLjk5MiwxOCwxMzguMzU3LDE4IHMxMjAuMzU3LDUzLjk5MiwxMjAuMzU3LDEyMC4zNTdTMjA0LjcyMywyNTguNzE1LDEzOC4zNTcsMjU4LjcxNXoiLz4gPHBhdGggZD0iTTE5NC43OTgsMTYwLjkwM2MtNC4xODgtMi42NzctOS43NTMtMS40NTQtMTIuNDMyLDIuNzMyYy04LjY5NCwxMy41OTMtMjMuNTAzLDIxLjcwOC0zOS42MTQsMjEuNzA4IGMtMjUuOTA4LDAtNDYuOTg1LTIxLjA3OC00Ni45ODUtNDYuOTg2czIxLjA3Ny00Ni45ODYsNDYuOTg1LTQ2Ljk4NmMxNS42MzMsMCwzMC4yLDcuNzQ3LDM4Ljk2OCwyMC43MjMgYzIuNzgyLDQuMTE3LDguMzc1LDUuMjAxLDEyLjQ5NiwyLjQxOGM0LjExOC0yLjc4Miw1LjIwMS04LjM3NywyLjQxOC0xMi40OTZjLTEyLjExOC0xNy45MzctMzIuMjYyLTI4LjY0NS01My44ODItMjguNjQ1IGMtMzUuODMzLDAtNjQuOTg1LDI5LjE1Mi02NC45ODUsNjQuOTg2czI5LjE1Miw2NC45ODYsNjQuOTg1LDY0Ljk4NmMyMi4yODEsMCw0Mi43NTktMTEuMjE4LDU0Ljc3OC0zMC4wMDkgQzIwMC4yMDgsMTY5LjE0NywxOTguOTg1LDE2My41ODIsMTk0Ljc5OCwxNjAuOTAzeiIvPiA8L2c+IDwvZz4KDTwvc3ZnPg==" height="22">

[<img alt="Discord" src="https://img.shields.io/discord/835936528140206122?style=for-the-badge&logo=discord&logoColor=white&label=Discord&color=7289da" height="22">][discord]

</div>

## Introduction

`smol-bytes` provides Rust byte buffers with inline storage for small values.
For ordinary copying and static constructors, values up to 62 bytes start
inline. Imported or owner-backed values, and retained shared heap views, can
remain heap-backed even when their current length is 62 bytes or less. Larger
or growable values can use `bytes`-backed heap storage. The crate also provides
mutable buffers and wrappers that guarantee valid UTF-8.

There are currently no published releases on crates.io, PyPI, or npm. All
packages in this repository build from source.

## Rust

```toml
[dependencies]
smol-bytes = "0.1"
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

`smol-bytes` is under the terms of both the MIT license and the
Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

Copyright (c) 2026 Al Liu.

[Github-url]: https://github.com/al8n/smol-bytes/
[CI-url]: https://github.com/al8n/smol-bytes/actions/workflows/ci.yml
[doc-url]: https://docs.rs/smol-bytes
[crates-url]: https://crates.io/crates/smol-bytes
[codecov-url]: https://app.codecov.io/gh/al8n/smol-bytes/
[discord]: https://discord.gg/ujPyPY6ud9
