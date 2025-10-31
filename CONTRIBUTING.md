# Contributing to smol-bytes

Thank you for your interest in contributing to `smol-bytes`! This document provides guidelines and information for contributors.

## Design Philosophy

`smol-bytes` is optimized for scenarios where:
1. **Most data is small** (≤62 bytes)
2. **Frequent cloning** is required
3. **Minimal allocations** are critical

### Primary Use Cases

- **Language Lexers & Parsers**: Fast token cloning for AST building
- **Protocol Parsers**: Efficient small message handling
- **FFI Boundaries**: Minimal allocations crossing language barriers
- **Concurrent Compilation**: Cheap token sharing across threads

## Architecture Overview

### Inline Storage (62 bytes)

The core optimization is storing small data inline on the stack:
- **Size**: 62 bytes maximum for inline data
- **Storage**: `[MaybeUninit<u8>; 62]` array
- **Metadata**: 1 byte for length, 1 byte for cursor position
- **Total**: 64 bytes struct size

### Two ImmutableStorage Patterns

#### `shared::Bytes`
- **Goal**: Fast `bytes::Bytes` interop
- **Heap behavior**: Preserves allocations for zero-copy conversions
- **Best for**: Lexers/parsers with existing `bytes` integration

#### `compact::Bytes`
- **Goal**: Minimize memory usage
- **Heap behavior**: Aggressively converts heap→inline when data shrinks
- **Best for**: Memory-constrained environments

### Split API Design

The split methods return `Result<BytesMut, Buffer>`:
- `Ok(BytesMut)`: Heap buffer that can grow
- `Err(Buffer)`: Inline buffer (max 62 bytes, mutable)

This design ensures:
- Consistent truncation semantics
- Type-safe distinction between heap/inline
- No data loss (tail always returned)

## Development Setup

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/al8n/smol-bytes.git
cd smol-bytes
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test --package smol-bytes

# Run doc tests
cargo test --doc

# Run benchmarks
cargo bench
```

### Code Style

- Follow standard Rust formatting: `cargo fmt`
- Ensure no warnings: `cargo clippy -- -D warnings`
- Add documentation for all public APIs
- Include examples in doc comments

## Making Changes

### Adding Features

When adding new features:
1. Ensure it aligns with the design philosophy (small data, fast cloning)
2. Add comprehensive tests
3. Update documentation
4. Add benchmarks if performance-critical

### Performance Considerations

- **Inline path**: Must be optimized for `memcpy` speed
- **Heap path**: Should minimize reference count operations
- **No-std**: Ensure changes work without std

### Testing Requirements

All changes must include:
- ✅ Unit tests
- ✅ Doc tests (examples in documentation)
- ✅ Edge cases (empty buffers, boundary conditions)
- ✅ Property tests (when applicable)

## API Design Guidelines

### Naming Conventions
- `is_inline()`, `is_heap()`: State queries
- `make_heap()`: Explicit conversions
- `split_off()`, `split_to()`: Consume and return parts

### Error Handling
- Use `Result` for operations that may fail
- Use `Option` for operations that may not be applicable
- Panic for programmer errors (bounds violations)

### Documentation
- Start with one-line summary
- Explain inline vs heap behavior
- Provide concrete examples
- Note performance characteristics
- Mention FFI considerations

## FFI Bindings (Future)

### Python (pyo3)
When implementing Python bindings:
- Expose `Bytes` as Python bytes-like object
- Handle GIL appropriately
- Document memory ownership semantics

### WebAssembly (wasm-bindgen)
When implementing WASM bindings:
- Minimize allocations across boundary
- Use inline storage to avoid malloc
- Document JS interop patterns

## Benchmarking

### Running Benchmarks

```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench -- clone

# Quick mode
cargo bench -- --quick
```

### Adding Benchmarks

When adding benchmarks:
- Compare inline vs heap performance
- Test different data sizes (8, 16, 32, 62, 64, 128 bytes)
- Measure clone, creation, and drop costs

## Documentation

### Doc Comments

```rust
/// Creates a new buffer from a static byte slice.
///
/// This is a `const` function that stores data inline if it fits (≤62 bytes).
/// For lexers/parsers, use this with `const` keyword literals.
///
/// ## Examples
///
/// ```rust
/// use smol_bytes::shared::Bytes;
///
/// // Keywords fit inline - zero allocation
/// const KEYWORD_FN: Bytes = Bytes::from_static(b"fn");
/// ```
pub const fn from_static(bytes: &'static [u8]) -> Self {
    // ...
}
```

### README Updates

When making significant changes:
- Update README.md with new capabilities
- Add examples for new use cases
- Update performance characteristics table

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Run full test suite
4. Run benchmarks to ensure no regressions
5. Update documentation
6. Create release tag

## Community

- **Issues**: Report bugs or request features
- **Discussions**: Ask questions or propose ideas
- **Pull Requests**: Submit changes with tests and documentation

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).

## Questions?

Feel free to open an issue or start a discussion!
