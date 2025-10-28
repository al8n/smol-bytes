# smol-bytes

`smol-bytes` 提供 `SmolBytes` 类型，它能够将最多 39 字节的数据存储在栈上，
超出部分则使用 [`bytes::Bytes`] 自动共享堆内存。对内联数据的克隆是简单的拷贝，
对堆数据的克隆则是引用计数递增。

## 特性
- 支持 `no_std`（需启用 `alloc` 特性）
- 内联与 `Bytes` 数据的克隆均为 `O(1)`
- 提供可选的 `serde`、`borsh`、`arbitrary` 集成
- 提供构建器 API 方便按需构造字节序列

## 使用示例

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

## 许可证

本项目同时采用

- [Apache License 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

两种开源许可证。请选择其中任意一种遵守。
