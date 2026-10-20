"""Static typing contract for the public Python API."""

from typing import SupportsIndex

from smol_bytes import (
    Buffer,
    Bytes,
    BytesMut,
    SliceAssignment,
    Utf8Buffer,
    Utf8Bytes,
    Utf8BytesMut,
)
from smol_bytes.compact import Bytes as CompactBytes
from smol_bytes.compact import Utf8Bytes as CompactUtf8Bytes
from smol_bytes.shared import Bytes as SharedBytes
from smol_bytes.shared import Utf8Bytes as SharedUtf8Bytes


class Index:
    def __init__(self, value: int) -> None:
        self.value = value

    def __index__(self) -> int:
        return self.value


def check_surface() -> None:
    raw: Bytes = Bytes.from_bytes(bytearray(b"raw"))
    alias: SharedBytes = raw
    mutable: BytesMut = alias.to_mutable()
    mutable.put_slice(memoryview(b"!"))
    compact: CompactBytes = mutable.freeze_compact()
    decoded: str = compact.decode_utf8()
    del decoded

    fixed: Buffer = Buffer.from_utf8("fixed")
    fixed.put_slice(bytearray(b"!"))
    scalar: SupportsIndex = Index(0)
    assignment: SliceAssignment = [Index(65), Index(66)]
    fixed[scalar] = Index(65)
    fixed[0:2] = assignment
    mutable[scalar] = Index(65)
    mutable[0:2] = (Index(value) for value in (66, 67))

    text: Utf8Bytes = Utf8Bytes.from_str("é🦀")
    text_alias: SharedUtf8Bytes = text
    mutable_text: Utf8BytesMut = text_alias.to_mutable()
    mutable_text.truncate_chars(1)
    mutable_text.insert_chars(scalar, "x")
    mutable_text.insert_str_chars(scalar, "é")
    mutable_text.replace_range_chars(scalar, scalar, "!")
    popped: str | None = mutable_text.pop()
    lossy_text: Utf8BytesMut = Utf8BytesMut.from_bytes_lossy(bytearray(b"\xff"))
    text_is_inline: bool = lossy_text.try_make_inline()
    compact_text: CompactUtf8Bytes = mutable_text.freeze_compact()
    selected: CompactUtf8Bytes = compact_text.slice_chars(0, 1)
    selected.advance_bytes(2)
    compact_lossy: CompactUtf8Bytes = CompactUtf8Bytes.from_bytes_lossy(memoryview(b"\xff"))
    immutable_pop: str | None = compact_lossy.pop()

    fixed_text: Utf8Buffer = Utf8Buffer.from_str("abc")
    fixed_text.split_to_chars(1)
    fixed_text.insert_chars(scalar, "x")
    fixed_text.insert_str_chars(scalar, "!")
    fixed_text.replace_range_chars(scalar, scalar, "?")
    fixed_pop: str | None = fixed_text.pop()
    raw_is_inline: bool = mutable.try_make_inline()

    del popped, text_is_inline, immutable_pop, fixed_pop, raw_is_inline
