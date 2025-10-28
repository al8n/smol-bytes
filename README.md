# smol-bytes

`smol-bytes` provides `SmolBytes`, a small byte buffer that stores up to 39 bytes
inline and falls back to a [`bytes::Bytes`] allocation for larger data. Cloning
inline values is a plain copy, while heap-backed buffers share the allocation.

## Highlights
- `no_std` friendly with an `alloc` feature
- `O(1)` clone for inline and `Bytes`-backed values
- Optional `serde`, `borsh`, and `arbitrary` integrations
- Builder API for constructing buffers incrementally

## Usage

Add the crate to your project:

```toml
[dependencies]
smol-bytes = "0.1"
```

```rust
use smol_bytes::SmolBytes;

let inline = SmolBytes::new(b"hello");
assert_eq!(inline.as_slice(), b"hello");
assert!(!inline.is_heap());

let large = SmolBytes::new(vec![42u8; 128]);
assert!(large.is_heap());
```

## Features

- `std` *(default)* – enables `std` support
- `alloc` – use the crate in `no_std` + `alloc`
- `serde` – serialization via `serde`
- `borsh` – serialization via `borsh`
- `arbitrary` – support for the `arbitrary` crate

## License

Licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Copyright (c) 2024
