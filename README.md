# smol-bytes

Small, clone-efficient Rust byte buffers with 62 bytes of inline storage.

This repository is unpublished: Rust, Python, and JavaScript bindings are
built from source. For production Rust use, pin a reviewed Git revision:

```toml
[dependencies]
smol-bytes = { git = "https://github.com/al8n/smol-bytes", rev = "<reviewed-revision>" }
```

## Storage and types

`Buffer` and `Utf8Buffer` are fixed inline values. `BytesMut` and
`Utf8BytesMut` grow into heap storage. The immutable aliases are:

| Type | Strategy | Mutable |
| --- | --- | :---: |
| `shared::Bytes` / `Bytes` | preserves heap storage and sharing | No |
| `compact::Bytes` | compacts short heap-backed values inline | No |
| `shared::Utf8Bytes` / `Utf8Bytes` | shared immutable UTF-8 | No |
| `compact::Utf8Bytes` | compact immutable UTF-8 | No |

Ordinary copies and static values of at most 62 bytes start inline. A shared
heap-backed slice remains heap-backed and shared even if it is non-empty and
at most 62 bytes; an inline source slice stays inline; empty values use the
canonical empty representation.

```rust
use smol_bytes::{shared, Buf};

let mut heap = shared::Bytes::from(vec![0_u8; 128]);
heap.advance(80);
let slice = heap.slice(0..32);
assert!(slice.is_heap());
```

## Features and MSRV

`std` is the default. `alloc` enables heap-backed types without `std`; `serde`,
`borsh`, `arbitrary`, and `quickcheck` enable their integrations. `pyo3`
implies `std`, as does `wasm`.

Rust 1.85 is the library and feature MSRV. Development-only test and benchmark
dependencies can require a newer compiler.

## Python from source

Python 3.11+, Rust, and maturin are required:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install maturin pytest
maturin develop --features pyo3 --manifest-path smol-bytes/Cargo.toml
python -m pytest tests/python -v
```

## JavaScript / WebAssembly from source

Install Node.js 20, `wasm-pack` 0.13.1, and `wasm32-unknown-unknown`, then run.
The generated package is pinned to wasm-bindgen 0.2.105 for reproducibility:

```bash
rustup target add wasm32-unknown-unknown
cd js
npm ci
npm run build
npm test
```

The Wasm build entry point is:

```bash
wasm-pack build smol-bytes --target bundler --out-dir ../js/pkg -- --features wasm
```

## Development

```bash
cargo fmt --all -- --check
cargo clippy --workspace --no-default-features --features std,alloc,serde,borsh,arbitrary,quickcheck --all-targets -- -D warnings
cargo test --workspace --no-default-features --features std,serde,borsh,arbitrary,quickcheck
cargo doc --package smol-bytes --no-deps
```

See [CONTRIBUTING.md](CONTRIBUTING.md). This project is available under either
the MIT license or the Apache License, Version 2.0, at your option. See
[LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
