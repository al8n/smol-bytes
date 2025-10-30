# smol-bytes

[![Crates.io](https://img.shields.io/crates/v/smol-bytes.svg)](https://crates.io/crates/smol-bytes)
[![Documentation](https://docs.rs/smol-bytes/badge.svg)](https://docs.rs/smol-bytes)
[![License](https://img.shields.io/crates/l/smol-bytes.svg)](https://github.com/al8n/smol-bytes)

High-performance, clone-efficient byte buffers optimized for small data (<62 bytes). Perfect for language lexers, parsers, and FFI boundaries where most tokens/chunks are small and frequent cloning is required.

## Why smol-bytes?

### 🚀 Blazing Fast Cloning
- **Inline storage**: Data ≤62 bytes is stored on the stack - cloning is a simple `memcpy`
- **Zero-cost abstraction**: No heap allocations for small buffers
- **O(1) clone**: Both inline and heap-backed buffers clone in constant time

### 🎯 Perfect for Lexers & Parsers
Most programming language tokens (identifiers, keywords, operators) are small:
- `let`, `fn`, `const`, `return` - all fit inline
- Most variable names are <62 characters
- Numbers, operators, punctuation - all inline
- **Result**: Near-zero allocation AST building with cheap token cloning for concurrent compilation

### 🔌 FFI-Friendly
- Minimal allocations make it ideal for crossing FFI boundaries
- Python bindings via `pyo3` (coming soon)
- WebAssembly support via `wasm-bindgen` (coming soon)
- `no_std` compatible with optional `alloc`

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
smol-bytes = "0.1"
```

### Basic Usage

```rust
use smol_bytes::Bytes;

// Small data (≤62 bytes) stored inline - no allocation!
let token = Bytes::from_static(b"identifier");
assert!(token.is_inline());

let cloned = token.clone(); // Just a memcpy, extremely fast
assert_eq!(token, cloned);

// Large data automatically uses heap
let large = Bytes::copy_from_slice(&[0u8; 128]);
assert!(large.is_heap());
let cloned_large = large.clone(); // O(1) - shares the allocation
```

### Lexer/Parser Example

```rust
use smol_bytes::shared::Bytes;

#[derive(Clone)]
enum Token {
    Identifier(Bytes),
    Keyword(Bytes),
    Number(Bytes),
}

// Tokens are cloned frequently during AST construction
// With smol-bytes, this is extremely efficient
fn parse_tokens(source: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();

    // Most identifiers fit inline (no allocation)
    let id = Bytes::copy_from_slice(b"variable_name");
    tokens.push(Token::Identifier(id.clone())); // Fast clone

    // Keywords definitely fit inline
    let kw = Bytes::from_static(b"fn");
    tokens.push(Token::Keyword(kw.clone())); // Fast clone

    tokens
}
```

## Two Optimization Strategies

Choose the strategy that fits your use case:

### `shared::Bytes` - Recommended for Most Use Cases
Fast bidirectional conversions with `bytes::Bytes`, preserves heap allocations:

```rust
use smol_bytes::shared::Bytes;

let data = Bytes::from_static(b"hello");
// Seamless interop with bytes::Bytes
let bytes_crate: bytes::Bytes = data.into();
```

### `compact::Bytes` - Maximum Memory Efficiency
Aggressively inlines data, minimizes memory footprint:

```rust
use smol_bytes::compact::Bytes;

let data = Bytes::from_static(b"compact");
// Optimized for memory usage over interop
```

## Mutable Buffers

`BytesMut` provides mutable byte buffers with the same inline optimization:

```rust
use smol_bytes::BytesMut;

let mut buf = BytesMut::new();
buf.extend_from_slice(b"hello");
assert!(buf.is_inline()); // Still on the stack

// Automatically promotes to heap when needed
buf.extend_from_slice(&[0u8; 128]);
assert!(buf.is_heap());
```

### Split Operations

Split operations return different types based on storage:

```rust
use smol_bytes::BytesMut;

let mut buf = BytesMut::from(&b"hello world"[..]);
match buf.split_off(5) {
    Ok(tail) => {
        // Heap: got BytesMut (can grow)
        // buf = "hello", tail = " world"
    }
    Err(tail) => {
        // Inline: got Buffer (max 62 bytes, still mutable)
        // buf = "hello", tail = " world"
    }
}
```

## Features

- **`std`** *(default)* – Standard library support
- **`alloc`** – Use in `no_std` environments with allocator
- **`serde`** – Serialization support via serde
- **`borsh`** – Serialization support via borsh
- **`arbitrary`** – Fuzzing support via arbitrary
- **`quickcheck`** – Property testing support

## Performance Characteristics

| Operation | Inline (≤62 bytes) | Heap (>62 bytes) |
|-----------|-------------------|------------------|
| Clone | `O(1)` memcpy | `O(1)` refcount |
| Create | Stack allocation | Heap allocation |
| Drop | No-op | Refcount decrement |
| Memory | 64 bytes on stack | Pointer + heap |

## Upcoming Features

- 🐍 **Python bindings** via `pyo3`
- 🕸️ **WebAssembly support** via `wasm-bindgen`
- 📦 Additional serialization formats

## Use Cases

✅ **Language Lexers & Parsers** - Fast token cloning for AST building
✅ **Protocol Parsers** - Efficient small message handling
✅ **Configuration Files** - Keys and small values
✅ **FFI Boundaries** - Minimal allocation crossing language barriers
✅ **Concurrent Compilation** - Cheap token cloning across threads

## Benchmarks

Run benchmarks with:

```bash
cargo bench
```

Inline cloning is typically 2-3x faster than heap-allocated buffers, with the added benefit of zero allocations.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

**Note**: This crate is designed for scenarios where most data is small (≤62 bytes). If your data is consistently large, consider using [`bytes`](https://crates.io/crates/bytes) directly.
