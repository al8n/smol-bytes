"""Comprehensive pytest suite for the smol-bytes pyo3 bindings."""

import copy
import gc
import pickle
import struct

import pytest

# ---------------------------------------------------------------------------
# 1. Import paths
# ---------------------------------------------------------------------------

from smol_bytes import Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut
from smol_bytes.shared import Bytes as SharedBytes
from smol_bytes.compact import Bytes as CompactBytes
from smol_bytes.compact import Utf8Bytes as CompactUtf8Bytes


class TestImportPaths:
    """Verify all 7 types import from their documented modules."""

    def test_buffer_import(self):
        assert Buffer is not None

    def test_bytes_mut_import(self):
        assert BytesMut is not None

    def test_utf8_buffer_import(self):
        assert Utf8Buffer is not None

    def test_utf8_bytes_import(self):
        assert Utf8Bytes is not None

    def test_utf8_bytes_mut_import(self):
        assert Utf8BytesMut is not None

    def test_shared_bytes_import(self):
        assert SharedBytes is not None

    def test_compact_bytes_import(self):
        assert CompactBytes is not None


# ---------------------------------------------------------------------------
# 2. Construction
# ---------------------------------------------------------------------------


class TestConstruction:
    """Test constructors for all types."""

    def test_buffer_empty(self):
        buf = Buffer()
        assert len(buf) == 0

    def test_buffer_from_bytes(self):
        buf = Buffer.from_bytes(b"hello")
        assert bytes(buf) == b"hello"

    def test_buffer_from_str(self):
        buf = Buffer.from_str("hello")
        assert bytes(buf) == b"hello"

    def test_buffer_from_bytes_overflow(self):
        with pytest.raises(BufferError):
            Buffer.from_bytes(b"x" * 63)

    def test_bytes_mut_empty(self):
        buf = BytesMut()
        assert len(buf) == 0

    def test_bytes_mut_from_bytes(self):
        buf = BytesMut.from_bytes(b"hello")
        assert bytes(buf) == b"hello"

    def test_bytes_mut_from_str(self):
        buf = BytesMut.from_str("hello")
        assert bytes(buf) == b"hello"

    def test_bytes_mut_with_capacity(self):
        buf = BytesMut.with_capacity(100)
        assert len(buf) == 0
        assert buf.capacity() >= 100

    def test_bytes_mut_zeroed(self):
        buf = BytesMut.zeroed(5)
        assert bytes(buf) == b"\x00\x00\x00\x00\x00"
        assert len(buf) == 5

    def test_shared_bytes_empty(self):
        b = SharedBytes()
        assert len(b) == 0

    def test_shared_bytes_from_bytes(self):
        b = SharedBytes.from_bytes(b"hello")
        assert bytes(b) == b"hello"

    def test_shared_bytes_from_str(self):
        b = SharedBytes.from_str("hello")
        assert bytes(b) == b"hello"

    def test_compact_bytes_empty(self):
        b = CompactBytes()
        assert len(b) == 0

    def test_compact_bytes_from_bytes(self):
        b = CompactBytes.from_bytes(b"hello")
        assert bytes(b) == b"hello"

    def test_compact_bytes_from_str(self):
        b = CompactBytes.from_str("hello")
        assert bytes(b) == b"hello"

    def test_utf8_buffer_empty(self):
        buf = Utf8Buffer()
        assert len(buf) == 0
        assert str(buf) == ""

    def test_utf8_buffer_from_str(self):
        buf = Utf8Buffer.from_str("hello")
        assert str(buf) == "hello"

    def test_utf8_buffer_from_str_overflow(self):
        with pytest.raises(ValueError):
            Utf8Buffer.from_str("x" * 63)

    def test_utf8_bytes_empty(self):
        b = Utf8Bytes()
        assert len(b) == 0

    def test_utf8_bytes_from_str(self):
        b = Utf8Bytes.from_str("hello")
        assert str(b) == "hello"

    def test_utf8_bytes_mut_empty(self):
        buf = Utf8BytesMut()
        assert len(buf) == 0

    def test_utf8_bytes_mut_from_str(self):
        buf = Utf8BytesMut.from_str("hello")
        assert str(buf) == "hello"

    def test_utf8_bytes_mut_with_capacity(self):
        buf = Utf8BytesMut.with_capacity(100)
        assert len(buf) == 0
        assert buf.capacity() >= 100


# ---------------------------------------------------------------------------
# 3. Basic operations: bytes(), str(), len(), bool()
# ---------------------------------------------------------------------------


class TestBasicOperations:
    """Test __bytes__, __str__, __len__, __bool__."""

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"hi"),
            lambda: BytesMut.from_bytes(b"hi"),
            lambda: SharedBytes.from_bytes(b"hi"),
            lambda: CompactBytes.from_bytes(b"hi"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_bytes_returns_bytes(self, make_buf):
        assert bytes(make_buf()) == b"hi"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("hi"),
            lambda: Utf8Bytes.from_str("hi"),
            lambda: Utf8BytesMut.from_str("hi"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_str_returns_str(self, make_buf):
        assert str(make_buf()) == "hi"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"abc"),
            lambda: BytesMut.from_bytes(b"abc"),
            lambda: SharedBytes.from_bytes(b"abc"),
            lambda: CompactBytes.from_bytes(b"abc"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_len_byte_types(self, make_buf):
        assert len(make_buf()) == 3

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("abc"),
            lambda: Utf8Bytes.from_str("abc"),
            lambda: Utf8BytesMut.from_str("abc"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_len_utf8_types(self, make_buf):
        assert len(make_buf()) == 3

    @pytest.mark.parametrize(
        "make_empty",
        [
            Buffer,
            BytesMut,
            SharedBytes,
            CompactBytes,
            Utf8Buffer,
            Utf8Bytes,
            Utf8BytesMut,
        ],
        ids=[
            "Buffer",
            "BytesMut",
            "SharedBytes",
            "CompactBytes",
            "Utf8Buffer",
            "Utf8Bytes",
            "Utf8BytesMut",
        ],
    )
    def test_bool_empty_is_false(self, make_empty):
        assert not bool(make_empty())

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"x"),
            lambda: BytesMut.from_bytes(b"x"),
            lambda: SharedBytes.from_bytes(b"x"),
            lambda: CompactBytes.from_bytes(b"x"),
            lambda: Utf8Buffer.from_str("x"),
            lambda: Utf8Bytes.from_str("x"),
            lambda: Utf8BytesMut.from_str("x"),
        ],
        ids=[
            "Buffer",
            "BytesMut",
            "SharedBytes",
            "CompactBytes",
            "Utf8Buffer",
            "Utf8Bytes",
            "Utf8BytesMut",
        ],
    )
    def test_bool_nonempty_is_true(self, make_buf):
        assert bool(make_buf())


@pytest.mark.parametrize(
    "bytes_type",
    [SharedBytes, CompactBytes],
    ids=["shared.Bytes", "compact.Bytes"],
)
@pytest.mark.parametrize(
    "mutation",
    [
        pytest.param(lambda owner: owner.advance(17), id="advance"),
        pytest.param(lambda owner: owner.clear(), id="clear"),
        pytest.param(lambda owner: owner.split_to(31), id="split_to"),
        pytest.param(lambda owner: owner.split_off(31), id="split_off"),
        pytest.param(lambda owner: owner.get_u64_le(), id="get_u64_le"),
        pytest.param(lambda owner: owner.get_uint(7), id="get_uint"),
    ],
)
def test_memoryview_remains_snapshot_after_owner_mutation(bytes_type, mutation):
    original = bytes(range(96))
    owner = bytes_type.from_bytes(original)
    view = memoryview(owner)

    assert type(view.obj) is bytes
    assert view.obj is not owner
    assert view.obj is not original
    assert view.obj == original
    snapshot_id = id(view.obj)

    returned = mutation(owner)
    del owner
    del returned

    for value in range(64):
        allocation = bytes_type.from_bytes(bytes([value]) * len(original))
        del allocation
    gc.collect()

    assert bytes(view) == original
    assert view[0] == original[0]
    assert view[-1] == original[-1]
    assert bytes(view[11:37]) == original[11:37]
    assert view.readonly is True
    assert view.format == "B"
    assert view.itemsize == 1
    assert view.ndim == 1
    assert view.shape == (len(original),)
    assert view.strides == (1,)
    assert view.c_contiguous is True
    assert type(view.obj) is bytes
    assert id(view.obj) == snapshot_id
    assert view.obj == original
    with pytest.raises(TypeError):
        view[0] = 255


# ---------------------------------------------------------------------------
# 4. Pickle round-trip
# ---------------------------------------------------------------------------


class TestPickle:
    """Test pickle.dumps / pickle.loads round-trip for all types."""

    @pytest.mark.parametrize(
        "make_buf, check",
        [
            (lambda: Buffer.from_bytes(b"hello"), lambda b: bytes(b) == b"hello"),
            (lambda: BytesMut.from_bytes(b"hello"), lambda b: bytes(b) == b"hello"),
            (lambda: SharedBytes.from_bytes(b"hello"), lambda b: bytes(b) == b"hello"),
            (lambda: CompactBytes.from_bytes(b"hello"), lambda b: bytes(b) == b"hello"),
            (lambda: Utf8Buffer.from_str("hello"), lambda b: str(b) == "hello"),
            (lambda: Utf8Bytes.from_str("hello"), lambda b: str(b) == "hello"),
            (lambda: Utf8BytesMut.from_str("hello"), lambda b: str(b) == "hello"),
        ],
        ids=[
            "Buffer",
            "BytesMut",
            "SharedBytes",
            "CompactBytes",
            "Utf8Buffer",
            "Utf8Bytes",
            "Utf8BytesMut",
        ],
    )
    def test_pickle_round_trip(self, make_buf, check):
        original = make_buf()
        restored = pickle.loads(pickle.dumps(original))
        assert check(restored)
        assert type(restored) is type(original)


# ---------------------------------------------------------------------------
# 5. Hash
# ---------------------------------------------------------------------------


class TestHash:
    """Test __hash__ on immutable types and TypeError on mutable types."""

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: SharedBytes.from_bytes(b"hello"),
            lambda: CompactBytes.from_bytes(b"hello"),
            lambda: Utf8Bytes.from_str("hello"),
        ],
        ids=["SharedBytes", "CompactBytes", "Utf8Bytes"],
    )
    def test_hash_works_on_immutable(self, make_buf):
        obj = make_buf()
        h = hash(obj)
        assert isinstance(h, int)

    def test_hash_consistent_for_same_content(self):
        a = SharedBytes.from_bytes(b"test")
        b = SharedBytes.from_bytes(b"test")
        assert hash(a) == hash(b)

    def test_hash_consistent_utf8(self):
        a = Utf8Bytes.from_str("test")
        b = Utf8Bytes.from_str("test")
        assert hash(a) == hash(b)

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: BytesMut.from_bytes(b"hello"),
            lambda: Utf8Buffer.from_str("hello"),
            lambda: Utf8BytesMut.from_str("hello"),
        ],
        ids=["BytesMut", "Utf8Buffer", "Utf8BytesMut"],
    )
    def test_hash_raises_on_mutable(self, make_buf):
        with pytest.raises(TypeError):
            hash(make_buf())

    def test_buffer_has_hash(self):
        """Buffer is a special case -- it implements __hash__ despite being mutable."""
        buf = Buffer.from_bytes(b"hello")
        h = hash(buf)
        assert isinstance(h, int)


# ---------------------------------------------------------------------------
# 6. Iteration
# ---------------------------------------------------------------------------


class TestIteration:
    """Byte types yield ints, UTF-8 types yield single-char strs."""

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"abc"),
            lambda: BytesMut.from_bytes(b"abc"),
            lambda: SharedBytes.from_bytes(b"abc"),
            lambda: CompactBytes.from_bytes(b"abc"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_byte_iteration_yields_ints(self, make_buf):
        result = list(make_buf())
        assert result == [97, 98, 99]
        assert all(isinstance(x, int) for x in result)

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("abc"),
            lambda: Utf8Bytes.from_str("abc"),
            lambda: Utf8BytesMut.from_str("abc"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_utf8_iteration_yields_chars(self, make_buf):
        result = list(make_buf())
        assert result == ["a", "b", "c"]
        assert all(isinstance(x, str) and len(x) == 1 for x in result)

    def test_utf8_iteration_multibyte(self):
        buf = Utf8Bytes.from_str("cafe\u0301")  # cafe with combining accent
        chars = list(buf)
        assert chars == ["c", "a", "f", "e", "\u0301"]

    def test_utf8_iteration_emoji(self):
        buf = Utf8BytesMut.from_str("a\U0001f980b")  # a + crab emoji + b
        chars = list(buf)
        assert chars == ["a", "\U0001f980", "b"]


# ---------------------------------------------------------------------------
# 7. Slicing
# ---------------------------------------------------------------------------


class TestSlicing:
    """Test __getitem__ with ints and slices."""

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"hello"),
            lambda: BytesMut.from_bytes(b"hello"),
            lambda: SharedBytes.from_bytes(b"hello"),
            lambda: CompactBytes.from_bytes(b"hello"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_byte_getitem_int(self, make_buf):
        buf = make_buf()
        assert buf[0] == ord("h")
        assert buf[-1] == ord("o")

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"hello world"),
            lambda: BytesMut.from_bytes(b"hello world"),
            lambda: SharedBytes.from_bytes(b"hello world"),
            lambda: CompactBytes.from_bytes(b"hello world"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_byte_getitem_slice(self, make_buf):
        buf = make_buf()
        assert buf[2:5] == b"llo"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"abcdef"),
            lambda: BytesMut.from_bytes(b"abcdef"),
            lambda: SharedBytes.from_bytes(b"abcdef"),
            lambda: CompactBytes.from_bytes(b"abcdef"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_byte_getitem_step(self, make_buf):
        buf = make_buf()
        assert buf[::2] == b"ace"

    def test_byte_index_out_of_range(self):
        buf = Buffer.from_bytes(b"hi")
        with pytest.raises(IndexError):
            buf[5]

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("hello"),
            lambda: Utf8Bytes.from_str("hello"),
            lambda: Utf8BytesMut.from_str("hello"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_utf8_getitem_int(self, make_buf):
        buf = make_buf()
        assert buf[0] == "h"
        assert buf[-1] == "o"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("hello"),
            lambda: Utf8Bytes.from_str("hello"),
            lambda: Utf8BytesMut.from_str("hello"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_utf8_getitem_slice(self, make_buf):
        buf = make_buf()
        assert buf[1:4] == "ell"

    def test_utf8_index_out_of_range(self):
        buf = Utf8Bytes.from_str("hi")
        with pytest.raises(IndexError):
            buf[5]


# ---------------------------------------------------------------------------
# 8. Split operations
# ---------------------------------------------------------------------------


class TestSplitOperations:
    """Test split_to and split_off on byte and UTF-8 types."""

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"hello world"),
            lambda: BytesMut.from_bytes(b"hello world"),
            lambda: SharedBytes.from_bytes(b"hello world"),
            lambda: CompactBytes.from_bytes(b"hello world"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_byte_split_to(self, make_buf):
        buf = make_buf()
        head = buf.split_to(5)
        assert bytes(head) == b"hello"
        assert bytes(buf) == b" world"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"hello world"),
            lambda: BytesMut.from_bytes(b"hello world"),
            lambda: SharedBytes.from_bytes(b"hello world"),
            lambda: CompactBytes.from_bytes(b"hello world"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_byte_split_off(self, make_buf):
        buf = make_buf()
        tail = buf.split_off(6)
        assert bytes(buf) == b"hello "
        assert bytes(tail) == b"world"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("hello world"),
            lambda: Utf8Bytes.from_str("hello world"),
            lambda: Utf8BytesMut.from_str("hello world"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_utf8_split_to(self, make_buf):
        buf = make_buf()
        head = buf.split_to(5)
        assert str(head) == "hello"
        assert str(buf) == " world"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("hello world"),
            lambda: Utf8Bytes.from_str("hello world"),
            lambda: Utf8BytesMut.from_str("hello world"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_utf8_split_off(self, make_buf):
        buf = make_buf()
        tail = buf.split_off(6)
        assert str(buf) == "hello "
        assert str(tail) == "world"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("caf\u00e9"),
            lambda: Utf8Bytes.from_str("caf\u00e9"),
            lambda: Utf8BytesMut.from_str("caf\u00e9"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_utf8_split_on_char_boundary(self, make_buf):
        # "cafe" is 5 bytes: c(1) a(1) f(1) e-acute(2)
        buf = make_buf()
        head = buf.split_to(3)
        assert str(head) == "caf"
        assert str(buf) == "\u00e9"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("caf\u00e9"),
            lambda: Utf8Bytes.from_str("caf\u00e9"),
            lambda: Utf8BytesMut.from_str("caf\u00e9"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_utf8_split_mid_char_raises(self, make_buf):
        # "cafe" byte 4 is in the middle of the 2-byte e-acute
        buf = make_buf()
        with pytest.raises(ValueError):
            buf.split_to(4)

    def test_split_out_of_bounds(self):
        buf = Buffer.from_bytes(b"hi")
        with pytest.raises(IndexError):
            buf.split_to(10)


# ---------------------------------------------------------------------------
# 9. Rich comparison
# ---------------------------------------------------------------------------


class TestRichComparison:
    """Test ==, !=, <, > between same types and with bytes/str."""

    def test_byte_eq_same_type(self):
        a = SharedBytes.from_bytes(b"hello")
        b = SharedBytes.from_bytes(b"hello")
        assert a == b

    def test_byte_ne_same_type(self):
        a = SharedBytes.from_bytes(b"hello")
        b = SharedBytes.from_bytes(b"world")
        assert a != b

    def test_byte_lt_same_type(self):
        a = CompactBytes.from_bytes(b"abc")
        b = CompactBytes.from_bytes(b"abd")
        assert a < b

    def test_byte_gt_same_type(self):
        a = CompactBytes.from_bytes(b"abd")
        b = CompactBytes.from_bytes(b"abc")
        assert a > b

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"hello"),
            lambda: BytesMut.from_bytes(b"hello"),
            lambda: SharedBytes.from_bytes(b"hello"),
            lambda: CompactBytes.from_bytes(b"hello"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_byte_eq_with_python_bytes(self, make_buf):
        buf = make_buf()
        assert buf == b"hello"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"hello"),
            lambda: BytesMut.from_bytes(b"hello"),
            lambda: SharedBytes.from_bytes(b"hello"),
            lambda: CompactBytes.from_bytes(b"hello"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    def test_byte_ne_with_python_bytes(self, make_buf):
        buf = make_buf()
        assert buf != b"world"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("hello"),
            lambda: Utf8Bytes.from_str("hello"),
            lambda: Utf8BytesMut.from_str("hello"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_utf8_eq_with_python_str(self, make_buf):
        buf = make_buf()
        assert buf == "hello"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("hello"),
            lambda: Utf8Bytes.from_str("hello"),
            lambda: Utf8BytesMut.from_str("hello"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_utf8_ne_with_python_str(self, make_buf):
        buf = make_buf()
        assert buf != "world"

    def test_utf8_lt_with_python_str(self):
        buf = Utf8Bytes.from_str("abc")
        assert buf < "abd"

    def test_utf8_gt_with_python_str(self):
        buf = Utf8Bytes.from_str("abd")
        assert buf > "abc"


# ---------------------------------------------------------------------------
# 10. __bytes__ on UTF-8 types
# ---------------------------------------------------------------------------


class TestBytesOnUtf8:
    """Test that __bytes__ returns correct UTF-8 encoding."""

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("hello"),
            lambda: Utf8Bytes.from_str("hello"),
            lambda: Utf8BytesMut.from_str("hello"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_bytes_ascii(self, make_buf):
        assert bytes(make_buf()) == b"hello"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("caf\u00e9"),
            lambda: Utf8Bytes.from_str("caf\u00e9"),
            lambda: Utf8BytesMut.from_str("caf\u00e9"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_bytes_multibyte(self, make_buf):
        expected = "caf\u00e9".encode("utf-8")
        assert bytes(make_buf()) == expected

    def test_bytes_emoji(self):
        buf = Utf8Bytes.from_str("\U0001f980")
        assert bytes(buf) == "\U0001f980".encode("utf-8")
        assert len(bytes(buf)) == 4

    def test_bytes_empty(self):
        buf = Utf8Bytes()
        assert bytes(buf) == b""


# ---------------------------------------------------------------------------
# 11. Storage info (is_inline / is_heap)
# ---------------------------------------------------------------------------


class TestStorageInfo:
    """Test is_inline() and is_heap() for types that support it."""

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: SharedBytes.from_bytes(b"small"),
            lambda: CompactBytes.from_bytes(b"small"),
            lambda: BytesMut.from_bytes(b"small"),
            lambda: Utf8Bytes.from_str("small"),
            lambda: Utf8BytesMut.from_str("small"),
        ],
        ids=["SharedBytes", "CompactBytes", "BytesMut", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_small_is_inline(self, make_buf):
        buf = make_buf()
        assert buf.is_inline() is True
        assert buf.is_heap() is False

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: SharedBytes.from_bytes(b"x" * 100),
            lambda: CompactBytes.from_bytes(b"x" * 100),
            lambda: BytesMut.from_bytes(b"x" * 100),
            lambda: Utf8Bytes.from_str("x" * 100),
            lambda: Utf8BytesMut.from_str("x" * 100),
        ],
        ids=["SharedBytes", "CompactBytes", "BytesMut", "Utf8Bytes", "Utf8BytesMut"],
    )
    def test_large_is_heap(self, make_buf):
        buf = make_buf()
        assert buf.is_inline() is False
        assert buf.is_heap() is True


# ---------------------------------------------------------------------------
# 12. Copy support
# ---------------------------------------------------------------------------


class TestCopySupport:
    """Test copy.copy() and copy.deepcopy() produce equal but independent objects."""

    @pytest.mark.parametrize(
        "make_buf, to_val",
        [
            (lambda: BytesMut.from_bytes(b"hello"), bytes),
            (lambda: SharedBytes.from_bytes(b"hello"), bytes),
            (lambda: CompactBytes.from_bytes(b"hello"), bytes),
            (lambda: Utf8Buffer.from_str("hello"), str),
            (lambda: Utf8Bytes.from_str("hello"), str),
            (lambda: Utf8BytesMut.from_str("hello"), str),
        ],
        ids=[
            "BytesMut",
            "SharedBytes",
            "CompactBytes",
            "Utf8Buffer",
            "Utf8Bytes",
            "Utf8BytesMut",
        ],
    )
    def test_copy_produces_equal(self, make_buf, to_val):
        original = make_buf()
        copied = copy.copy(original)
        assert to_val(copied) == to_val(original)
        assert type(copied) is type(original)

    @pytest.mark.parametrize(
        "make_buf, to_val",
        [
            (lambda: BytesMut.from_bytes(b"hello"), bytes),
            (lambda: SharedBytes.from_bytes(b"hello"), bytes),
            (lambda: CompactBytes.from_bytes(b"hello"), bytes),
            (lambda: Utf8Buffer.from_str("hello"), str),
            (lambda: Utf8Bytes.from_str("hello"), str),
            (lambda: Utf8BytesMut.from_str("hello"), str),
        ],
        ids=[
            "BytesMut",
            "SharedBytes",
            "CompactBytes",
            "Utf8Buffer",
            "Utf8Bytes",
            "Utf8BytesMut",
        ],
    )
    def test_deepcopy_produces_equal(self, make_buf, to_val):
        original = make_buf()
        copied = copy.deepcopy(original)
        assert to_val(copied) == to_val(original)
        assert type(copied) is type(original)

    def test_copy_independent_bytes_mut(self):
        """Mutating the copy should not affect the original."""
        original = BytesMut.from_bytes(b"hello")
        copied = copy.copy(original)
        copied.clear()
        assert bytes(original) == b"hello"
        assert bytes(copied) == b""

    def test_deepcopy_independent_utf8_bytes_mut(self):
        """Mutating the deepcopy should not affect the original."""
        original = Utf8BytesMut.from_str("hello")
        copied = copy.deepcopy(original)
        copied.push_str(" world")
        assert str(original) == "hello"
        assert str(copied) == "hello world"


# ---------------------------------------------------------------------------
# Additional coverage: get/put integer methods
# ---------------------------------------------------------------------------


class TestGetPutIntegers:
    """Test get_* and put_* methods for reading/writing integers."""

    def test_buffer_put_get_u16_be(self):
        buf = Buffer()
        buf.put_u16(0x1234)
        assert buf[0] == 0x12
        assert buf[1] == 0x34
        val = buf.get_u16()
        assert val == 0x1234

    def test_buffer_put_get_u16_le(self):
        buf = Buffer()
        buf.put_u16_le(0x1234)
        assert buf[0] == 0x34
        assert buf[1] == 0x12
        val = buf.get_u16_le()
        assert val == 0x1234

    def test_bytes_mut_put_get_u32(self):
        buf = BytesMut()
        buf.put_u32(0xDEADBEEF)
        val = buf.get_u32()
        assert val == 0xDEADBEEF

    def test_bytes_mut_put_get_u32_le(self):
        buf = BytesMut()
        buf.put_u32_le(0xDEADBEEF)
        val = buf.get_u32_le()
        assert val == 0xDEADBEEF

    def test_shared_bytes_get_u8(self):
        b = SharedBytes.from_bytes(b"\x42\x43")
        assert b.get_u8() == 0x42
        assert b.get_u8() == 0x43

    def test_get_on_empty_raises(self):
        buf = Buffer()
        with pytest.raises(BufferError):
            buf.get_u8()

    def test_buffer_put_get_f64(self):
        buf = Buffer()
        buf.put_f64(3.14)
        val = buf.get_f64()
        assert abs(val - 3.14) < 1e-10

    def test_bytes_mut_put_i64_le(self):
        buf = BytesMut()
        buf.put_i64_le(-12345)
        val = buf.get_i64_le()
        assert val == -12345

    def test_buffer_put_overflow_raises(self):
        """Buffer has a 62-byte cap; putting too many bytes should fail."""
        buf = Buffer()
        # 62 bytes fills capacity for a Buffer
        for _ in range(62):
            buf.put_u8(0xFF)
        with pytest.raises(BufferError):
            buf.put_u8(0x00)


# ---------------------------------------------------------------------------
# Additional: UTF-8 push/push_str
# ---------------------------------------------------------------------------


class TestUtf8PushOperations:
    """Test push(ch) and push_str(s) on mutable UTF-8 types."""

    def test_utf8_buffer_push_char(self):
        buf = Utf8Buffer()
        buf.push("a")
        buf.push("b")
        assert str(buf) == "ab"

    def test_utf8_buffer_push_str(self):
        buf = Utf8Buffer()
        buf.push_str("hello")
        assert str(buf) == "hello"

    def test_utf8_bytes_mut_push_multibyte(self):
        buf = Utf8BytesMut()
        buf.push_str("caf")
        buf.push("\u00e9")
        assert str(buf) == "caf\u00e9"
        assert bytes(buf) == "caf\u00e9".encode("utf-8")

    def test_utf8_bytes_mut_push_emoji(self):
        buf = Utf8BytesMut()
        buf.push("\U0001f980")
        assert str(buf) == "\U0001f980"

    def test_utf8_buffer_push_str_overflow(self):
        buf = Utf8Buffer()
        with pytest.raises(ValueError):
            buf.push_str("x" * 63)


# ---------------------------------------------------------------------------
# Additional: multi-byte UTF-8 content
# ---------------------------------------------------------------------------


class TestMultibyteUtf8:
    """Test multi-byte UTF-8 content (cafe, emoji, etc.)."""

    def test_cafe_len_is_bytes(self):
        buf = Utf8Bytes.from_str("caf\u00e9")
        assert len(buf) == 5  # 3 ASCII + 2-byte e-acute

    def test_crab_emoji_len(self):
        buf = Utf8BytesMut.from_str("\U0001f980")
        assert len(buf) == 4  # 4-byte crab emoji

    def test_cafe_iteration(self):
        buf = Utf8Buffer.from_str("caf\u00e9")
        chars = list(buf)
        assert chars == ["c", "a", "f", "\u00e9"]

    def test_emoji_roundtrip_pickle(self):
        original = Utf8Bytes.from_str("\U0001f980 Rust")
        restored = pickle.loads(pickle.dumps(original))
        assert str(restored) == "\U0001f980 Rust"

    def test_multibyte_bytes_encoding(self):
        buf = Utf8BytesMut.from_str("\u00e9\U0001f980")
        raw = bytes(buf)
        assert raw == "\u00e9\U0001f980".encode("utf-8")
        assert len(raw) == 6  # 2 + 4

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("\u00e9x"),
            lambda: Utf8Bytes.from_str("\u00e9x"),
            lambda: CompactUtf8Bytes.from_str("\u00e9x"),
            lambda: Utf8BytesMut.from_str("\u00e9x"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "CompactUtf8Bytes", "Utf8BytesMut"],
    )
    def test_byte_advance_requires_char_boundary(self, make_buf):
        buf = make_buf()

        with pytest.raises(BufferError, match="UTF-8 character boundary"):
            buf.advance(1)

        assert str(buf) == "\u00e9x"
        buf.advance(2)
        assert str(buf) == "x"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("\u00e9x"),
            lambda: Utf8Bytes.from_str("\u00e9x"),
            lambda: CompactUtf8Bytes.from_str("\u00e9x"),
            lambda: Utf8BytesMut.from_str("\u00e9x"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "CompactUtf8Bytes", "Utf8BytesMut"],
    )
    def test_numeric_reads_require_char_boundary(self, make_buf):
        buf = make_buf()

        with pytest.raises(BufferError, match="UTF-8 character boundary"):
            buf.get_u8()
        with pytest.raises(BufferError, match="UTF-8 character boundary"):
            buf.get_uint(1)

        assert str(buf) == "\u00e9x"
        assert buf.get_u16() == int.from_bytes("\u00e9".encode(), "big")
        assert str(buf) == "x"


# ---------------------------------------------------------------------------
# Additional: __contains__
# ---------------------------------------------------------------------------


class TestContains:
    """Test the __contains__ (in operator) for types."""

    def test_byte_contains_int(self):
        buf = Buffer.from_bytes(b"hello")
        assert ord("h") in buf
        assert 0xFF not in buf

    def test_byte_contains_bytes(self):
        buf = SharedBytes.from_bytes(b"hello world")
        assert b"world" in buf
        assert b"xyz" not in buf

    def test_utf8_contains_str(self):
        buf = Utf8Bytes.from_str("hello world")
        assert "world" in buf

    def test_utf8_contains_empty(self):
        buf = Utf8BytesMut.from_str("hello")
        assert "" in buf


# ---------------------------------------------------------------------------
# Additional: repr
# ---------------------------------------------------------------------------


class TestRepr:
    """Test __repr__ returns a string."""

    def test_buffer_repr(self):
        buf = Buffer.from_bytes(b"hi")
        r = repr(buf)
        assert isinstance(r, str)

    def test_utf8_bytes_repr(self):
        buf = Utf8Bytes.from_str("hi")
        r = repr(buf)
        assert isinstance(r, str)


# ---------------------------------------------------------------------------
# Additional: clear / truncate
# ---------------------------------------------------------------------------


class TestClearTruncate:
    """Test clear() and truncate() methods."""

    def test_buffer_clear(self):
        buf = Utf8Buffer.from_str("hello")
        buf.clear()
        assert len(buf) == 0
        assert str(buf) == ""

    def test_bytes_mut_clear(self):
        buf = BytesMut.from_bytes(b"hello")
        buf.clear()
        assert len(buf) == 0

    def test_shared_bytes_truncate(self):
        buf = SharedBytes.from_bytes(b"hello world")
        buf.truncate(5)
        assert bytes(buf) == b"hello"

    def test_shared_bytes_clear(self):
        buf = SharedBytes.from_bytes(b"hello")
        buf.clear()
        assert len(buf) == 0

    def test_bytes_mut_truncate(self):
        buf = BytesMut.from_bytes(b"hello world")
        buf.truncate(5)
        assert bytes(buf) == b"hello"
