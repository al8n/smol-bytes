"""Focused tests for the public Python API contract."""

from __future__ import annotations

from array import array
from pathlib import Path
import pickle

import pytest

import smol_bytes
from smol_bytes import (
    Buffer,
    Bytes,
    BytesMut,
    ReadableBuffer,
    SliceAssignment,
    Utf8Buffer,
    Utf8Bytes,
    Utf8BytesMut,
)
from smol_bytes.compact import Bytes as CompactBytes
from smol_bytes.compact import Utf8Bytes as CompactUtf8Bytes
from smol_bytes.shared import Bytes as SharedBytes
from smol_bytes.shared import Utf8Bytes as SharedUtf8Bytes


RAW_TYPES = (Buffer, BytesMut, Bytes, CompactBytes)
UTF8_TYPES = (Utf8Buffer, Utf8Bytes, Utf8BytesMut, CompactUtf8Bytes)
RAW_FACTORIES = tuple(cls.from_bytes for cls in RAW_TYPES)


def test_root_shared_aliases_keep_root_pickle_identity() -> None:
    assert Bytes is SharedBytes
    assert Utf8Bytes is SharedUtf8Bytes
    assert Bytes.__module__ == "smol_bytes"
    assert Utf8Bytes.__module__ == "smol_bytes"
    assert CompactBytes is not Bytes
    assert CompactUtf8Bytes is not Utf8Bytes

    assert pickle.loads(b"csmol_bytes.shared\nBytes\n.") is Bytes
    assert pickle.loads(b"csmol_bytes.shared\nUtf8Bytes\n.") is Utf8Bytes
    assert pickle.loads(b"csmol_bytes._smol_bytes.shared\nBytes\n.") is Bytes
    assert pickle.loads(b"csmol_bytes._smol_bytes.shared\nUtf8Bytes\n.") is Utf8Bytes
    assert type(pickle.loads(pickle.dumps(Bytes.from_bytes(b"pickle")))) is Bytes


def test_typed_marker_and_stub_alias_contract() -> None:
    package = Path(smol_bytes.__file__).parent
    assert (package / "py.typed").is_file()
    root_stub = package / "__init__.pyi"
    shared_stub = package / "shared" / "__init__.pyi"
    for path in (root_stub, shared_stub, package / "compact" / "__init__.pyi"):
        compile(path.read_text(), str(path), "exec")
    assert "ReadableBuffer" in root_stub.read_text()
    assert "SliceAssignment" in root_stub.read_text()
    assert "from .. import Bytes as Bytes" in shared_stub.read_text()


def test_runtime_typing_aliases_and_keyword_parameters() -> None:
    assert "ReadableBuffer" in smol_bytes.__all__
    assert "SliceAssignment" in smol_bytes.__all__
    assert ReadableBuffer is smol_bytes.ReadableBuffer
    assert SliceAssignment is smol_bytes.SliceAssignment

    for cls in RAW_TYPES:
        assert bytes(cls.from_bytes(data=b"ab")) == b"ab"
        assert bytes(cls.from_utf8(s="ab")) == b"ab"

    for cls in (Buffer, BytesMut):
        value = cls.from_bytes(data=b"ab")
        value.put_slice(data=bytearray(b"cd"))
        value.put_bytes(value=ord("e"), count=1)
        value.put_u8(value=ord("f"))
        value.__setitem__(0, ord("A"))
        value.__setitem__(slice(1, 3), range(ord("B"), ord("D")))
        assert bytes(value) == b"ABCdef"


@pytest.mark.parametrize("cls", (Buffer, BytesMut))
def test_raw_assignment_uses_python_index_protocol(cls: type[object]) -> None:
    class Index:
        def __init__(self, value: int) -> None:
            self.value = value

        def __index__(self) -> int:
            return self.value

    value = cls.from_bytes(b"abcd")
    value[Index(0)] = Index(ord("A"))
    value[1:3] = [Index(ord("B")), Index(ord("C"))]
    assert bytes(value) == b"ABCd"


@pytest.mark.parametrize("cls", RAW_TYPES)
def test_raw_text_renames_are_explicit_and_str_is_total(cls: type[object]) -> None:
    assert not hasattr(cls, "from_str")
    assert not hasattr(cls, "to_string")
    assert cls.from_utf8("café").decode_utf8() == "café"
    invalid = cls.from_bytes(b"\xff")
    assert str(invalid) == str(bytes(invalid))
    with pytest.raises(UnicodeDecodeError):
        invalid.decode_utf8()


@pytest.mark.parametrize("cls", RAW_TYPES)
@pytest.mark.parametrize(
    "factory",
    [
        lambda: b"abcdef",
        lambda: bytearray(b"abcdef"),
        lambda: memoryview(b"abcdef"),
        lambda: memoryview(array("b", [97, 98, 99, 100, 101, 102])),
    ],
)
def test_raw_buffer_inputs_are_readable_snapshots(cls: type[object], factory) -> None:
    source = factory()
    value = cls.from_bytes(source)
    assert bytes(value) == b"abcdef"
    if isinstance(source, bytearray):
        source[0] = ord("X")
        assert bytes(value) == b"abcdef"

    for bad, error in (
        (memoryview(b"abcdef")[::2], BufferError),
        (memoryview(array("H", [1, 2])), BufferError),
        (object(), TypeError),
    ):
        with pytest.raises(error):
            cls.from_bytes(bad)
    released = memoryview(bytearray(b"x"))
    released.release()
    with pytest.raises(BufferError):
        cls.from_bytes(released)


@pytest.mark.parametrize("cls", RAW_TYPES)
@pytest.mark.parametrize(
    ("typecode", "values", "expected"),
    [
        ("b", [-128, -1, 0, 1, 127], b"\x80\xff\x00\x01\x7f"),
        ("B", [0, 1, 127, 255], b"\x00\x01\x7f\xff"),
    ],
)
def test_raw_signed_and_unsigned_buffers_preserve_byte_values(
    cls: type[object], typecode: str, values: list[int], expected: bytes
) -> None:
    assert bytes(cls.from_bytes(array(typecode, values))) == expected


@pytest.mark.parametrize("cls", RAW_TYPES)
@pytest.mark.parametrize("typecode", ["b", "B"])
def test_raw_empty_signed_and_unsigned_buffers_are_supported(cls: type[object], typecode: str) -> None:
    assert bytes(cls.from_bytes(array(typecode))) == b""


@pytest.mark.parametrize("cls", RAW_TYPES)
@pytest.mark.parametrize("typecode", ["b", "B"])
def test_raw_contiguous_memoryview_subviews_are_snapshots(cls: type[object], typecode: str) -> None:
    source = array(typecode, [-3, -2, -1, 0, 1, 2] if typecode == "b" else [253, 254, 255, 0, 1, 2])
    assert bytes(cls.from_bytes(memoryview(source)[1:5])) == b"\xfe\xff\x00\x01"


@pytest.mark.parametrize("cls", RAW_TYPES)
@pytest.mark.parametrize("typecode", ["b", "B"])
def test_raw_signed_and_unsigned_owner_mutation_does_not_change_snapshot(
    cls: type[object], typecode: str
) -> None:
    source = array(typecode, [1, 2, 3])
    value = cls.from_bytes(source)
    source[0] = -7 if typecode == "b" else 247
    assert bytes(value) == b"\x01\x02\x03"


@pytest.mark.parametrize("cls", (Buffer, BytesMut))
def test_put_slice_buffer_matrix_and_fixed_failure_are_atomic(cls: type[object]) -> None:
    value = cls.from_bytes(b"ab")
    for data in (bytearray(b"c"), memoryview(b"d"), memoryview(array("b", [101])), b"f"):
        value.put_slice(data)
    assert bytes(value) == b"abcdef"

    fixed = Buffer.from_bytes(b"x" * 60)
    before = bytes(fixed)
    with pytest.raises(BufferError):
        fixed.put_slice(b"abc")
    assert bytes(fixed) == before
    with pytest.raises(BufferError):
        fixed.put_slice(memoryview(b"abcdef")[::2])
    assert bytes(fixed) == before


def test_mutable_slices_and_freezes_are_independent_and_reusable() -> None:
    raw = BytesMut.from_bytes(b"abcdef")
    copied = raw.slice(1, 4)
    copied[0] = ord("X")
    assert bytes(copied) == b"Xcd"
    assert bytes(raw) == b"abcdef"

    shared = raw.freeze_shared()
    assert bytes(raw) == b""
    raw.put_slice(b"reuse")
    clone = shared.to_mutable()
    clone[0] = ord("A")
    assert bytes(shared) == b"abcdef"
    assert bytes(clone) == b"Abcdef"

    text = Utf8BytesMut.from_str("é🦀x")
    copied_text = text.slice_chars(1, 2)
    copied_text.push_str("!")
    assert str(copied_text) == "🦀!"
    assert str(text) == "é🦀x"
    compact = text.freeze_compact()
    assert str(text) == ""
    mutable = compact.to_mutable()
    mutable.push_str("!")
    assert str(compact) == "é🦀x"
    assert str(mutable) == "é🦀x!"


@pytest.mark.parametrize("make", RAW_FACTORIES, ids=["Buffer", "BytesMut", "Bytes", "CompactBytes"])
def test_raw_offset_exception_normalization(make) -> None:
    def fresh():
        return make(data=b"abcd")

    for method, keyword in (
        ("advance", "cnt"),
        ("truncate", "new_len"),
        ("split_to", "at"),
        ("split_off", "at"),
    ):
        value = fresh()
        with pytest.raises(IndexError):
            getattr(value, method)(**{keyword: -1})
        assert bytes(value) == b"abcd"

        value = fresh()
        with pytest.raises(OverflowError):
            getattr(value, method)(**{keyword: 10**100})
        assert bytes(value) == b"abcd"

    for start, end, error in (
        (-1, 1, IndexError),
        (0, 10**100, OverflowError),
        (3, 1, ValueError),
        (0, 5, IndexError),
    ):
        value = fresh()
        with pytest.raises(error):
            value.slice(start=start, end=end)
        assert bytes(value) == b"abcd"

    value = fresh()
    with pytest.raises(IndexError):
        value.split_to(at=5)
    assert bytes(value) == b"abcd"

    value = fresh()
    split_off_out_of_bounds = value.capacity() + 1 if isinstance(value, BytesMut) else 5
    with pytest.raises(IndexError):
        value.split_off(at=split_off_out_of_bounds)
    assert bytes(value) == b"abcd"

    value = fresh()
    value.truncate(new_len=5)
    assert bytes(value) == b"abcd"

    value = fresh()
    with pytest.raises(BufferError):
        value.advance(cnt=5)
    assert bytes(value) == b"abcd"


@pytest.mark.parametrize("cls", UTF8_TYPES)
def test_utf8_units_are_explicit_and_character_based(cls: type[object]) -> None:
    value = cls.from_str("é🦀e\u0301")
    assert len(value) == value.char_len() == 4
    assert value.byte_len() == value.remaining_bytes() == 9
    assert list(value) == ["é", "🦀", "e", "\u0301"]
    assert value[1:3] == "🦀e"
    for removed in ("split_to", "split_off", "slice", "truncate", "remaining", "advance"):
        assert not hasattr(value, removed)
    if cls is Utf8Buffer:
        assert not hasattr(value, "len")

    head = value.split_to_chars(2)
    assert str(head) == "é🦀"
    assert str(value) == "e\u0301"
    with pytest.raises(IndexError):
        value.split_off_chars(99)
    with pytest.raises(ValueError):
        value.slice_chars(1, 0)


@pytest.mark.parametrize("cls", (Utf8Buffer, Utf8BytesMut))
def test_utf8_mutable_unit_methods_and_capacity_errors(cls: type[object]) -> None:
    value = cls.from_str("é🦀x")
    value.truncate_chars(2)
    assert str(value) == "é🦀"
    value = cls.from_str("éx")
    with pytest.raises(ValueError):
        value.advance_bytes(1)
    assert str(value) == "éx"
    value.advance_bytes(2)
    assert str(value) == "x"

    if cls is Utf8Buffer:
        full = cls.from_str("x" * 62)
        with pytest.raises(BufferError):
            full.push("y")
        assert str(full) == "x" * 62


@pytest.mark.parametrize("cls", UTF8_TYPES)
def test_utf8_pop_returns_unicode_scalars_or_none(cls: type[object]) -> None:
    value = cls.from_str("e\u0301🦀")
    assert value.pop() == "🦀"
    assert value.pop() == "\u0301"
    assert value.pop() == "e"
    assert value.pop() is None


@pytest.mark.parametrize("cls", (Utf8Buffer, Utf8BytesMut))
def test_utf8_character_edits_accept_index_protocol_and_clean_keywords(cls: type[object]) -> None:
    class Index:
        def __init__(self, value: int) -> None:
            self.value = value

        def __index__(self) -> int:
            return self.value

    value = cls.from_str("éx")
    assert value.insert_chars(index=Index(1), ch="🦀") is None
    assert str(value) == "é🦀x"
    assert value.insert_str_chars(index=Index(3), s="!") is None
    assert str(value) == "é🦀x!"
    assert value.replace_range_chars(start=Index(1), end=Index(2), replacement="e\u0301") is None
    assert str(value) == "ée\u0301x!"

    for callback, error in (
        (lambda: value.insert_chars(index=-1, ch="!"), IndexError),
        (lambda: value.insert_chars(index=10**100, ch="!"), OverflowError),
        (lambda: value.insert_chars(index=99, ch="!"), IndexError),
        (lambda: value.replace_range_chars(start=3, end=1, replacement="!"), ValueError),
        (lambda: value.insert_chars(index=1, ch=1), TypeError),
        (lambda: value.insert_chars(index=1, ch=""), ValueError),
        (lambda: value.insert_chars(index=1, ch="xy"), ValueError),
        (lambda: value.insert_chars(index=1, ch="\ud800"), UnicodeEncodeError),
        (lambda: value.insert_str_chars(index=1, s="\ud800"), UnicodeEncodeError),
    ):
        before = str(value)
        with pytest.raises(error):
            callback()
        assert str(value) == before


def test_fixed_utf8_character_edit_capacity_failures_are_atomic() -> None:
    value = Utf8Buffer.from_str("x" * 62)
    with pytest.raises(BufferError):
        value.insert_chars(index=0, ch="!")
    assert str(value) == "x" * 62
    with pytest.raises(BufferError):
        value.replace_range_chars(start=0, end=0, replacement="!")
    assert str(value) == "x" * 62
    value.replace_range_chars(start=0, end=1, replacement="")
    assert str(value) == "x" * 61


@pytest.mark.parametrize("cls", (Utf8BytesMut, Utf8Bytes, CompactUtf8Bytes))
@pytest.mark.parametrize(
    "source",
    (
        b"hi\xff",
        bytearray(b"hi\xff"),
        memoryview(b"hi\xff"),
        memoryview(array("b", [104, 105, -1])),
    ),
)
def test_utf8_lossy_constructors_snapshot_supported_buffers(cls: type[object], source: object) -> None:
    assert str(cls.from_bytes_lossy(source)) == "hi�"

    for bad, error in (
        (memoryview(b"abc")[::2], BufferError),
        (memoryview(array("H", [1, 2])), BufferError),
        (object(), TypeError),
    ):
        with pytest.raises(error):
            cls.from_bytes_lossy(bad)


def test_mutable_values_can_try_to_compact_back_inline() -> None:
    raw = BytesMut.with_capacity(100)
    raw.put_slice(b"small")
    assert raw.is_heap()
    assert raw.try_make_inline()
    assert raw.is_inline()

    text = Utf8BytesMut.with_capacity(100)
    text.push_str("small")
    assert text.is_heap()
    assert text.try_make_inline()
    assert text.is_inline()

    large = Utf8BytesMut.from_str("x" * 63)
    assert not large.try_make_inline()
    assert large.is_heap()


def test_exception_taxonomy_and_variable_width_failures_are_atomic() -> None:
    with pytest.raises(UnicodeEncodeError):
        Bytes.from_utf8("\ud800")
    with pytest.raises(UnicodeEncodeError):
        Utf8Bytes.from_str("\ud800")
    with pytest.raises(OverflowError):
        Buffer().put_u8(256)

    raw = BytesMut.from_bytes(b"a")
    for width, error in ((-1, ValueError), (9, ValueError), (10**100, OverflowError), (2, BufferError)):
        with pytest.raises(error):
            raw.get_uint(width)
        assert bytes(raw) == b"a"

    text = Utf8Bytes.from_str("é")
    with pytest.raises(ValueError):
        text.get_u8()
    assert str(text) == "é"


def test_unsplit_is_non_consuming_and_self_alias_is_atomic() -> None:
    raw = BytesMut.from_bytes(b"a")
    other = BytesMut.from_bytes(b"b" * 100)
    assert raw.unsplit(other) is None
    assert bytes(raw) == b"a" + b"b" * 100
    assert bytes(other) == b"b" * 100
    with pytest.raises(RuntimeError):
        raw.unsplit(raw)

    text = Utf8BytesMut.from_str("a")
    other_text = Utf8BytesMut.from_str("🦀")
    assert text.unsplit(other_text) is None
    assert str(text) == "a🦀"
    assert str(other_text) == "🦀"
