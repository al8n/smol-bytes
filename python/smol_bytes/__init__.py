"""smol-bytes: High-performance, clone-efficient byte buffers."""

from collections.abc import Iterable
from typing import SupportsIndex, TypeAlias

from ._smol_bytes import Bytes, Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut

ReadableBuffer: TypeAlias = bytes | bytearray | memoryview
SliceAssignment: TypeAlias = ReadableBuffer | Iterable[SupportsIndex]

__all__ = [
    "ReadableBuffer",
    "SliceAssignment",
    "Bytes",
    "Buffer",
    "BytesMut",
    "Utf8Buffer",
    "Utf8Bytes",
    "Utf8BytesMut",
]
