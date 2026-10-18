<div align="center">
<h1>smol-bytes</h1>
</div>
<div align="center">

High-performance, clone-efficient byte buffers optimized for small data.

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

`smol-bytes` provides byte buffers that store up to 62 bytes inline — no heap
allocation, and cloning an inline value is a plain 64-byte copy. Larger values
use [`bytes`](https://crates.io/crates/bytes)-backed heap storage with
reference-counted clones. Two strategies control what happens when a heap
value shrinks back under the inline threshold, and UTF-8 wrappers layer
`String`-like, boundary-checked APIs over the same storage.

This is a good fit when most values are small and cloned often — tokens in a
lexer, keys and field names, protocol headers — and allocation pressure
matters.

## Install

```toml
[dependencies]
smol-bytes = "0.2"
```

Python (`pip install smol-bytes`) and JavaScript (`npm install smol-bytes`)
packages are also published; build-from-source instructions for each binding
are below.

## Quick start

The root `Bytes` type is an alias for `shared::Bytes`. Ordinary copies and
static values of at most 62 bytes start inline; imported or owner-backed
values and retained shared heap views can remain heap-backed below that
threshold.

```rust
use smol_bytes::{Bytes, Utf8Bytes};

let small = Bytes::from_static(b"identifier");
assert!(small.is_inline());

let cloned = small.clone(); // 64-byte copy, no allocation
assert_eq!(small, cloned);

let heap = Bytes::copy_from_slice(&[0_u8; 63]);
assert!(heap.is_heap());
assert_eq!(heap.len(), 63);

let text: Utf8Bytes = "identifier".parse().expect("infallible parse");
assert_eq!(text.as_str(), "identifier");
```

## Storage strategies

`shared::Bytes` preserves heap storage once a value lives there, keeping
conversions with `bytes::Bytes` zero-copy. `compact::Bytes` copies a shrinking
view of 62 bytes or fewer back into inline storage to release the allocation.

```rust
use smol_bytes::{compact, shared, Buf};

let mut shared = shared::Bytes::from(vec![0_u8; 100]);
let mut compact = compact::Bytes::from(vec![0_u8; 100]);

shared.advance(70);
compact.advance(70);

assert_eq!(shared.len(), 30);
assert_eq!(compact.len(), 30);
assert!(shared.is_heap());   // stays shareable and zero-copy convertible
assert!(compact.is_inline()); // allocation released, contents inlined
```

Rule of thumb: use `shared` (the default) for I/O and `bytes` interop; use
`compact` when memory footprint matters more than conversion speed.

## Types

| Type | Storage | Mutable | Purpose |
| --- | --- | :---: | --- |
| `Buffer` | Fixed inline bytes, up to 62 bytes | Yes | `no_std` fixed buffer |
| `Bytes` / `shared::Bytes` | Inline or shared heap-backed bytes | No | Immutable shared view |
| `compact::Bytes` | Inline or compacting heap-backed bytes | No | Immutable compacting view |
| `BytesMut` | Inline, then growable `bytes::BytesMut` storage | Yes | Mutable byte buffer |
| `Utf8Buffer` | Fixed inline, valid UTF-8 bytes | Yes | Small mutable UTF-8 value |
| `Utf8Bytes` / `compact::Utf8Bytes` | Shared or compacting UTF-8 bytes | No | Immutable UTF-8 value |
| `Utf8BytesMut` | Inline or growable valid UTF-8 bytes | Yes | Mutable UTF-8 value |

Every handle is 64 bytes. With the `bytes` dependency enabled, all byte types
implement `bytes::Buf`, and the mutable ones implement `bytes::BufMut`.

The immutable strategy types have immutable payloads but mutable views:
`advance`, `truncate`, and `clear` change only the visible cursor/window.
`Utf8Bytes` offers the same byte-indexed, UTF-8-boundary-checked view
operations and can reclaim unique full heap allocations with `try_into_mut`.
`Utf8Buffer` and `Utf8BytesMut` expose `as_mut_str`, `AsMut<str>`,
`BorrowMut<str>`, `DerefMut<Target = str>`, and mutable string-range indexing;
the standard `str` rules preserve UTF-8 validity.

`BytesMut::split_to`, `split_off`, and `split` always return `BytesMut`.
Inline results stay inline; splitting an inline value's reserved-but-unwritten
capacity promotes it to heap storage so capacity partitioning remains exact.
`try_split_to` and `try_split_off` return `Result<BytesMut, OutOfBounds>` and
leave the receiver unchanged on bounds failure. `unsplit` always consumes and
appends its argument, using the zero-copy `bytes::BytesMut` path for contiguous
heap halves and copying/promoting only when required.

Rust UTF-8 split and slice indices are byte offsets that must fall on
character boundaries; offenders panic, and the `try_split_to`,
`try_split_off`, and `try_slice` variants return errors instead. (The Python
bindings differ deliberately — see below.)

The UTF-8 wrappers implement `FromStr`. Heap-capable wrappers parse
infallibly; `Utf8Buffer` returns `TryPutError` when text exceeds its fixed
inline capacity.

## 0.2 API migration

0.2 intentionally removes several ambiguous or lossy APIs.
For complete Rust, Python, and JavaScript examples, see
[the 0.2 migration guide](https://github.com/al8n/smol-bytes/blob/main/MIGRATING-0.2.md).

- Replace `Utf8Buffer::from(text)` with `Utf8Buffer::try_from(text)?` (or
  `Utf8Buffer::try_from_str(text)?`). `FromStr` remains available. This makes
  the 62-byte fixed-capacity limit explicit.
- `BytesMut` split methods now return `BytesMut` directly. Remove an inner
  `match`/`unwrap` around `split`, `split_to`, and `split_off`; use the
  corresponding `try_split_*` method when bounds must be reported instead of
  panicking. `try_split` is removed, and `unsplit` now returns `()`.
- Use `Buffer::resize(new_len, value)` and
  `Buffer::try_resize(new_len, value)` to choose the byte written during
  extension. Failed `try_resize` calls are atomic.
- UTF-8 index operations now use `Utf8IndexError`, preserving whether an
  invalid character boundary, index, or full range caused the failure.
  Converting owned `Buffer`, `BytesMut`, or raw strategy bytes into a UTF-8
  wrapper returns `FromUtf8Error<B>` on invalid input; call `into_bytes()` to
  recover the original allocation and view.
- Variable-width inherent `Buffer::try_get_uint{,_le,_ne}` and
  `Buffer::try_get_int{,_le,_ne}` methods now
  return `TryGetIntegerError`, distinguishing an invalid width from missing
  bytes. The `bytes::Buf` trait methods retain `bytes`' own error type and
  panic contract.
- `Utf8Buf` and `Utf8BufMut` are sealed implementation traits. Generic mutable
  code can use `try_push`/`try_push_str` and each type's `CapacityError`;
  `Utf8Buffer` reports `TryPutError`, while `Utf8BytesMut` uses `Infallible`.

## `bytes` interop

Conversions with the `bytes` crate are zero-copy wherever the representation
allows:

- `bytes::Bytes -> shared::Bytes` shares the allocation (`From` impl).
- `shared::Bytes -> bytes::Bytes` reuses heap backing; inline values copy.
- `compact::Bytes::from(bytes::Bytes)` inlines payloads of at most 62 bytes
  and shares larger ones.
- `BytesMut::freeze_shared` / `freeze_compact` convert without copying heap
  contents; `BytesMut::freeze` is the root-default shared conversion.
- `Utf8BytesMut::freeze` similarly returns the root-default `Utf8Bytes`;
  `freeze_shared` and `freeze_compact` remain available for explicit targets.
- Consuming `shared::Bytes`/`compact::Bytes` and UTF-8 strategy conversions
  apply the target strategy: large heap values keep their allocation and small
  values compact when converted to the compact strategy.
- `Bytes::try_into_mut` and `Utf8Bytes::try_into_mut` reclaim unique full heap
  allocations; a shared or sliced view is returned unchanged on failure.

```rust
use smol_bytes::{BytesMut, Utf8Bytes, Utf8BytesMut, compact};

let bytes = BytesMut::from(&b"payload"[..]).freeze();
assert_eq!(bytes, b"payload"[..]);

let mut text = Utf8Bytes::from("header: value");
text.advance(8);
assert_eq!(text.as_str(), "value"); // only this view moved

let compact: compact::Bytes = bytes.into();
assert_eq!(compact, b"payload"[..]);

let frozen = Utf8BytesMut::from("done").freeze();
assert_eq!(frozen.as_str(), "done");
```

## Features and MSRV

| Feature | Description |
| --- | --- |
| `std` (default) | Enables `alloc`, `bytes/std`, and Serde's `std`/`result` support |
| `alloc` | Heap-backed types and the optional `bytes` dependency without `std` |
| `bytes` | Compatibility alias for `alloc` |
| `serde` | Serde support |
| `schemars` | Schemars v1 schemas; UTF-8 wrappers are strings and byte wrappers serialize as `array<u8>`; implies `alloc` |
| `borsh` | Borsh support |
| `arbitrary` | `arbitrary` support for generated values |
| `quickcheck` | QuickCheck support; implies `std` |
| `async-graphql` | `Bytes` and `String` GraphQL scalars for `Bytes` and `Utf8Bytes`; implies `std` |
| `sqlx` | `sqlx` `Type`/`Encode`/`Decode` for `Bytes` and `Utf8Bytes`; implies `std` |
| `pyo3` | Python bindings; implies `std` |
| `wasm` | WebAssembly bindings; implies `std` |

With no features enabled the crate is `no_std` and provides the fixed
`Buffer` and `Utf8Buffer` types. `bytes::Buf`/`BufMut` are available once
`alloc`, `std`, or the compatibility `bytes` feature is selected; `alloc` adds
the heap-backed types without `std`.

Byte wrapper JSON schemas describe serialization as `array<u8>` and accept an
array or string under Schemars' deserialization contract. `Buffer` additionally
sets `maxItems` to `INLINE_CAP`; UTF-8 wrappers remain ordinary string schemas.

The `sqlx` bindings decode by borrowing from the row, so a value of at most 62
bytes is built inline with no heap allocation at all. That costs one thing:
PostgreSQL will not lend `BYTEA` out in a simple (unprepared) query, so code on
that path — `raw_sql`, or a SQL string handed straight to an `Executor` — has
to decode the byte types as `Vec<u8>` and convert. `query`, `query_as` and the
`query!` macros carry an argument list and are therefore prepared, so they are
unaffected, as are the UTF-8 types and every MySQL and SQLite path.

Rust 1.95 is the library MSRV, and the `bytes` dependency floor is 1.10.
Development-only test and benchmark dependencies can require a newer
compiler, and so do both optional integrations, to different floors:
`async-graphql` 7.2 declares Rust 1.89, and 7.2 is the floor because the 7.0
releases do not build against the 7.2 derive crate their own dependency range
admits; sqlx 0.9 declares Rust 1.94, and 0.9 is the floor because the impls are
written against the lifetime-free `Database::ArgumentBuffer` introduced there.
Enabling either raises the MSRV for the whole build.

## Verification

The test and CI story is deliberately heavier than the crate's size:

- Unit, integration, doc, and property tests (proptest state-machine
  comparisons against `Vec`/`String`, plus `quickcheck` and `arbitrary`
  generators that preserve type invariants).
- Miri over the full suite under both stacked borrows and tree borrows with
  strict provenance and symbolic alignment checks.
- Address, leak, memory, and thread sanitizers in CI.
- Deserialization is hardened: borsh reads length-prefixed payloads in
  bounded chunks instead of trusting the length prefix, and serde sequence
  hints are capped before preallocating.

## Python

Install the published package with `pip install smol-bytes` (Python 3.11+).

To build from a checkout instead, Python 3.11+, Rust, and `maturin` are
required:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install maturin pytest
maturin develop --features pyo3 --manifest-path smol-bytes-py/Cargo.toml
python -m pytest tests/python -v
```

The root `smol_bytes` module exposes canonical shared `Bytes` and `Utf8Bytes`
alongside `Buffer`, `BytesMut`, `Utf8Buffer`, and `Utf8BytesMut`.
`smol_bytes.shared.Bytes` and `smol_bytes.shared.Utf8Bytes` are aliases of
those exact root classes (including their root `__module__`), while
`smol_bytes.compact` remains a distinct compact strategy.

Raw byte classes use `from_utf8()` and `decode_utf8()` for explicit text
conversion; `str(raw)` is always the same total representation as
`str(bytes(raw))`. Their `from_bytes()` and mutable `put_slice()` methods
accept `bytes`, `bytearray`, and readable one-dimensional C-contiguous byte
`memoryview` values, copying the input before mutation.

The UTF-8 classes are string-like from Python: `len()`, indexing, and
iteration use Unicode scalar values. Their explicit APIs are `char_len()`,
`byte_len()`, `split_to_chars()`, `split_off_chars()`, `slice_chars()`, and
for mutable/fixed classes `truncate_chars()`. `remaining_bytes()` and
`advance_bytes()` are deliberately byte-oriented, as are `get_*` methods.

```python
from smol_bytes import Bytes, BytesMut, Utf8Bytes

raw = Bytes.from_bytes(b"abc")
assert raw.is_inline()
assert bytes(raw) == b"abc"
assert raw.decode_utf8() == "abc"

text = Utf8Bytes.from_str("café")
assert len(text) == 4        # Unicode characters
assert text.char_len() == 4
assert text.byte_len() == 5  # UTF-8 bytes
assert str(text) == "café"
assert str(text.split_to_chars(3)) == "caf"

mutable = BytesMut.from_bytes(bytearray(b"abc"))
frozen = mutable.freeze_shared()
assert bytes(mutable) == b""  # source remains reusable
assert bytes(frozen.to_mutable()) == b"abc"
```

Binding behavior worth knowing:

- Methods that allocate proportionally to caller data raise `MemoryError`
  on absurd or failing requests instead of aborting the interpreter, like
  CPython containers.
- `memoryview(...)` over the shared and compact `Bytes` classes exports a
  snapshot copy, not a live view.
- Slice assignment on the mutable classes requires matching lengths, and
  contiguous assignments take a direct copy fast path.
- Python `unsplit()` uses non-consuming snapshot semantics: it appends the
  call-entry contents while leaving the argument unchanged. Passing the same
  mutable object as both operands raises `RuntimeError` atomically.

## JavaScript / WebAssembly

Install the published package with `npm install smol-bytes`.

To build from a checkout instead, install Node.js 20, `wasm-pack` 0.13.1, and
the `wasm32-unknown-unknown` target. The generated package is pinned to
wasm-bindgen 0.2.127 for reproducibility:

```bash
rustup target add wasm32-unknown-unknown
cd js
npm ci
npm run build
npm test
```

The Wasm build entry point is:

```bash
wasm-pack build smol-bytes-wasm --target bundler --out-dir ../js/pkg --out-name smol_bytes -- --features wasm
```

The root export canonically contains `Bytes` and `Utf8Bytes` using the shared
strategy, alongside `Buffer`, `BytesMut`, `Utf8Buffer`, and `Utf8BytesMut`.
`smol-bytes/shared` exports the exact same immutable class objects; the
`smol-bytes/compact` entry point exports distinct compact classes. Every entry
point initializes iteration, disposal, and input validation independently.

Raw bytes use `fromUtf8()` and `decodeUtf8()`; they deliberately have no
string-like `fromString()` or raw `toString()` API. Neither raw nor UTF-8
public classes expose `fromStatic()`; UTF-8 classes retain
`fromString()` / `toString()` and make byte units explicit:
`byteLength()`, `charLength()`, `remainingBytes()`, `advanceBytes()`,
`splitToBytes()`, `splitOffBytes()`, `sliceBytes()`, and `truncateBytes()`.
Byte offsets must land at UTF-8 scalar boundaries. Numeric arguments are
checked before WebAssembly coercion, and malformed JavaScript UTF-16 strings
are rejected before encoding.

Byte conversions at the Wasm boundary return copies. `clone()` is available on
each public byte/string value class. Mutable `freezeShared()` / `freezeCompact()` drain their
source into the selected immutable strategy, while immutable `toMutable()`
returns an independent mutable value. JavaScript `unsplit()` snapshots and
appends `other` without consuming it; self-aliasing deterministically doubles
the value.

```typescript
import { Bytes, BytesMut, Utf8Bytes } from "smol-bytes";
import { Bytes as CompactBytes } from "smol-bytes/compact";

const raw = CompactBytes.fromBytes(new Uint8Array([1, 2, 3]));
console.assert(raw.isInline());
console.assert(raw.toBytes()[2] === 3);

const shared = Bytes.fromUtf8("hello");
console.assert(shared.decodeUtf8() === "hello");

const mutable = BytesMut.fromUtf8("ready");
const frozen = mutable.freezeShared();
console.assert(mutable.isEmpty() && frozen.decodeUtf8() === "ready");

const text = Utf8Bytes.fromString("café");
console.assert(text.byteLength() === 5);
console.assert(text.charLength() === 4);
console.assert(text.toString() === "café");
```

## Performance characteristics

Structural properties of the representations:

- Values of at most 62 bytes construct inline with no backing allocation.
- Every handle is 64 bytes; `Option<Bytes>` is the same size as `Bytes`.
- Cloning an inline value copies 64 bytes; cloning a heap-backed immutable
  value bumps a reference count; cloning `BytesMut`/`Utf8BytesMut` copies
  contents.
- Shared heap-backed conversions to and from `bytes::Bytes` reuse the
  backing allocation.
- Compact storage copies at most 62 bytes when inlining a shrinking view.

Run the benchmark suite when investigating a change:

```bash
cargo bench
cargo bench --bench clone
cargo bench --bench split_to
```

## Development

These commands match the main CI checks:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --no-default-features --features std,alloc,serde,schemars,borsh,arbitrary,quickcheck --all-targets -- -D warnings
cargo test --workspace --no-default-features --features std,serde,schemars,borsh,arbitrary,quickcheck
cargo test --package smol-bytes --no-default-features --features alloc,quickcheck
cargo check --package smol-bytes --lib --no-default-features
cargo doc --package smol-bytes --no-deps
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidance.

## License

`smol-bytes` is available under either the MIT license or the Apache License,
Version 2.0, at your option. See [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE).

Copyright (c) 2026 Al Liu.

[Github-url]: https://github.com/al8n/smol-bytes/
[CI-url]: https://github.com/al8n/smol-bytes/actions/workflows/ci.yml
[doc-url]: https://docs.rs/smol-bytes
[crates-url]: https://crates.io/crates/smol-bytes
[codecov-url]: https://app.codecov.io/gh/al8n/smol-bytes/
[discord]: https://discord.gg/ujPyPY6ud9
