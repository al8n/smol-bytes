"""Comprehensive pytest suite for the smol-bytes pyo3 bindings."""

import copy
import gc
import operator
import pickle
import struct
import sys

import pytest

# ---------------------------------------------------------------------------
# 1. Import paths
# ---------------------------------------------------------------------------

from smol_bytes import Buffer, BytesMut, Utf8Buffer, Utf8Bytes, Utf8BytesMut
from smol_bytes.shared import Bytes as SharedBytes
from smol_bytes.shared import Utf8Bytes as SharedUtf8Bytes
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
            lambda: Buffer.from_bytes(b"fo\xd8o"),
            lambda: BytesMut.from_bytes(b"fo\xd8o"),
            lambda: SharedBytes.from_bytes(b"fo\xd8o"),
            lambda: CompactBytes.from_bytes(b"fo\xd8o"),
        ],
        ids=["Buffer", "BytesMut", "SharedBytes", "CompactBytes"],
    )
    @pytest.mark.parametrize("to_string", [False, True], ids=["str", "to_string"])
    def test_invalid_utf8_decode_error(self, make_buf, to_string):
        value = make_buf()

        with pytest.raises(UnicodeDecodeError) as exc_info:
            value.to_string() if to_string else str(value)

        error = exc_info.value
        assert error.encoding == "utf-8"
        assert error.object == b"fo\xd8o"
        assert error.start == 2
        assert error.end == 3
        assert error.reason == "invalid utf-8"

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
    """Every exposed wrapper can change its observable value and is unhashable."""

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Buffer.from_bytes(b"hello"),
            lambda: BytesMut.from_bytes(b"hello"),
            lambda: SharedBytes.from_bytes(b"hello"),
            lambda: CompactBytes.from_bytes(b"hello"),
            lambda: Utf8Buffer.from_str("hello"),
            lambda: Utf8Bytes.from_str("hello"),
            lambda: CompactUtf8Bytes.from_str("hello"),
            lambda: Utf8BytesMut.from_str("hello"),
        ],
        ids=[
            "Buffer",
            "BytesMut",
            "SharedBytes",
            "CompactBytes",
            "Utf8Buffer",
            "Utf8Bytes",
            "CompactUtf8Bytes",
            "Utf8BytesMut",
        ],
    )
    def test_value_changing_wrapper_is_unhashable(self, make_buf):
        with pytest.raises(TypeError):
            hash(make_buf())


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


class TestNativeProtocolDifferential:
    """Cross-wrapper Python sequence and comparison contracts."""

    RAW = [
        lambda value: Buffer.from_bytes(value),
        lambda value: BytesMut.from_bytes(value),
        lambda value: SharedBytes.from_bytes(value),
        lambda value: CompactBytes.from_bytes(value),
    ]
    UTF8 = [
        lambda value: Utf8Buffer.from_str(value),
        lambda value: Utf8Bytes.from_str(value),
        lambda value: SharedUtf8Bytes.from_str(value),
        lambda value: CompactUtf8Bytes.from_str(value),
        lambda value: Utf8BytesMut.from_str(value),
    ]

    @staticmethod
    def _value(make, raw):
        return make(b"abcdef") if raw else make("aé€🦀e\u0301z")

    @staticmethod
    def _snapshot(value, raw):
        return bytes(value) if raw else str(value)

    @pytest.mark.parametrize(
        "subscript",
        [
            0,
            -1,
            slice(None),
            slice(None, None, -1),
            slice(-99, 99, 2),
            slice(None, None, -2),
            slice(5, None, sys.maxsize),
            slice(5, None, -sys.maxsize),
            slice(4, 1, -2),
            slice(2, 0, -1),
            slice(4, 1, 2),
        ],
    )
    def test_raw_indexing_matches_bytes(self, subscript):
        value = b"abcdef"
        for make in self.RAW:
            assert make(value)[subscript] == value[subscript]

    @pytest.mark.parametrize(
        "subscript",
        [
            0,
            -1,
            slice(None),
            slice(None, None, -1),
            slice(-99, 99, 2),
            slice(None, None, -2),
            slice(5, None, sys.maxsize),
            slice(5, None, -sys.maxsize),
            slice(4, 1, -2),
            slice(2, 0, -1),
            slice(4, 1, 2),
        ],
    )
    def test_utf8_indexing_matches_str_char_semantics(self, subscript):
        value = "aé€🦀e\u0301z"
        for make in self.UTF8:
            assert make(value)[subscript] == value[subscript]

    def test_empty_positive_step_slice_is_empty_for_every_wrapper(self):
        for raw, wrappers in ((True, self.RAW), (False, self.UTF8)):
            native = b"abcdef" if raw else "aé€🦀e\u0301z"
            subscript = slice(-1, -100)
            assert native[subscript] == (b"" if raw else "")
            for make in wrappers:
                assert self._value(make, raw)[subscript] == native[subscript]

    def test_slice_differential_covers_extreme_bounds_and_steps(self):
        bounds = (None, -sys.maxsize, -100, -1, 0, 1, 100, sys.maxsize)
        steps = (1, 2, sys.maxsize, -1, -2, -sys.maxsize)

        for raw, wrappers in ((True, self.RAW), (False, self.UTF8)):
            native = b"abcdef" if raw else "aé€🦀e\u0301z"
            for start in bounds:
                for stop in bounds:
                    for step in steps:
                        subscript = slice(start, stop, step)
                        expected = native[subscript]
                        for make in wrappers:
                            assert self._value(make, raw)[subscript] == expected

    @pytest.mark.parametrize("make", RAW)
    def test_raw_index_errors_match_bytes(self, make):
        value = make(b"a")
        for subscript, error in [(2, IndexError), (10**100, IndexError), (object(), TypeError), (slice(None, None, 0), ValueError)]:
            with pytest.raises(error):
                _ = value[subscript]

    @pytest.mark.parametrize("make", UTF8)
    def test_utf8_index_errors_match_str(self, make):
        value = make("é")
        for subscript, error in [(2, IndexError), (10**100, IndexError), (object(), TypeError), (slice(None, None, 0), ValueError)]:
            with pytest.raises(error):
                _ = value[subscript]

    @pytest.mark.parametrize("make", RAW + UTF8)
    def test_index_protocol_values_match_native_sequences(self, make):
        class Index:
            def __init__(self, value):
                self.value = value

            def __index__(self):
                return self.value

        native = b"abc" if make in self.RAW else "aé🦀"
        value = make(native)
        assert value[Index(-1)] == native[-1]
        with pytest.raises(IndexError):
            _ = native[10**100]
        with pytest.raises(IndexError):
            _ = native[Index(10**100)]
        with pytest.raises(IndexError):
            _ = value[Index(10**100)]

    @pytest.mark.parametrize("make", RAW + UTF8)
    def test_builtin_integer_index_fast_path_matches_native_sequences(self, make):
        class IntSubclass(int):
            def __index__(self):
                raise RuntimeError("int subclasses use their integer value")

        native = b"abc" if make in self.RAW else "aé🦀"
        value = make(native)
        for index in (False, True, IntSubclass(0), IntSubclass(-1)):
            assert value[index] == native[index]
        with pytest.raises(IndexError):
            _ = value[IntSubclass(10**100)]

    @pytest.mark.parametrize("make", RAW + UTF8)
    def test_index_protocol_ignores_operator_index_monkeypatch(self, make, monkeypatch):
        calls = []

        class Index:
            def __index__(self):
                calls.append(None)
                return -1

        def unexpected_operator_index(value):
            raise AssertionError(f"operator.index called for {value!r}")

        monkeypatch.setattr(operator, "index", unexpected_operator_index)
        native = b"abc" if make in self.RAW else "aé🦀"
        value = make(native)
        assert value[0] == native[0]
        assert value[Index()] == native[-1]
        assert calls == [None]

    @pytest.mark.parametrize(
        "error_type, message",
        [
            (RuntimeError, "runtime-boom"),
            (TypeError, "type-boom"),
            (OverflowError, "overflow-boom"),
        ],
    )
    @pytest.mark.parametrize("make", RAW + UTF8)
    def test_index_protocol_exceptions_match_native_sequences(self, make, error_type, message):
        class RaisingIndex:
            def __index__(self):
                raise error_type(message)

        native = b"abc" if make in self.RAW else "aé🦀"
        with pytest.raises(error_type) as native_error:
            _ = native[RaisingIndex()]
        with pytest.raises(error_type) as wrapper_error:
            _ = make(native)[RaisingIndex()]
        assert str(wrapper_error.value) == str(native_error.value) == message

    @pytest.mark.parametrize(
        "error_type, message",
        [
            (RuntimeError, "runtime-boom"),
            (TypeError, "type-boom"),
            (OverflowError, "overflow-boom"),
        ],
    )
    @pytest.mark.parametrize("make", RAW[:2])
    def test_setitem_propagates_index_protocol_exception_without_mutation(
        self, make, error_type, message
    ):
        class RaisingIndex:
            def __index__(self):
                raise error_type(message)

        native = bytearray(b"abc")
        with pytest.raises(error_type) as native_error:
            native[RaisingIndex()] = 0
        value = make(b"abc")
        with pytest.raises(error_type) as wrapper_error:
            value[RaisingIndex()] = 0
        assert str(wrapper_error.value) == str(native_error.value) == message
        assert bytes(value) == b"abc"

    @pytest.mark.parametrize("make", RAW + UTF8)
    def test_invalid_index_protocol_result_remains_type_error(self, make):
        class InvalidIndex:
            def __index__(self):
                return "not an integer"

        native = b"abc" if make in self.RAW else "aé🦀"
        with pytest.raises(TypeError):
            _ = native[InvalidIndex()]
        with pytest.raises(TypeError):
            _ = make(native)[InvalidIndex()]

    @pytest.mark.parametrize("make", RAW + UTF8)
    @pytest.mark.parametrize("method", ["get_uint", "get_uint_le", "get_int", "get_int_le"])
    def test_variable_width_index_protocol_and_atomicity(self, make, method):
        class Index:
            def __init__(self, value):
                self.value = value

            def __index__(self):
                return self.value

        class InvalidIndex:
            def __index__(self):
                return "not an integer"

        class RaisingIndex:
            def __init__(self, error_type, message):
                self.error_type = error_type
                self.message = message

            def __index__(self):
                raise self.error_type(self.message)

        raw = make in self.RAW
        source = b"\x01abcdefgh" if raw else "abcdefgh"

        value = make(source)
        before = self._snapshot(value, raw)
        assert getattr(value, method)(Index(0)) == 0
        assert self._snapshot(value, raw) == before

        value = make(source)
        getattr(value, method)(Index(1))
        assert self._snapshot(value, raw) == before[1:]

        for argument, error_type, message in [
            (object(), TypeError, None),
            (InvalidIndex(), TypeError, None),
            (-1, ValueError, None),
            (9, ValueError, None),
            (10**100, ValueError, None),
            (Index(10**100), ValueError, None),
            (RaisingIndex(RuntimeError, "runtime-width"), RuntimeError, "runtime-width"),
            (RaisingIndex(TypeError, "type-width"), TypeError, "type-width"),
            (RaisingIndex(OverflowError, "overflow-width"), OverflowError, "overflow-width"),
        ]:
            value = make(source)
            before = self._snapshot(value, raw)
            with pytest.raises(error_type) as error:
                getattr(value, method)(argument)
            if message is not None:
                assert str(error.value) == message
            assert self._snapshot(value, raw) == before

    def test_cross_strategy_equality_and_native_domains(self):
        raw = [make(b"abc") for make in self.RAW]
        text = [make("abc") for make in self.UTF8]
        assert all(left == right for left in raw for right in raw)
        assert all(left == right for left in text for right in text)
        assert all(value != "abc" for value in raw)
        assert all(value != b"abc" for value in text)

    @pytest.mark.parametrize("make", RAW[:2])
    def test_raw_slice_assignment_matches_fixed_size_bytearray(self, make):
        cases = [
            (slice(4, 1, -2), b"XY"),
            (slice(2, 0, -1), b"XY"),
            (slice(None, None, -1), b"UVWXYZ"),
            (slice(4, 1, 2), b""),
            (slice(5, None, sys.maxsize), b"X"),
        ]
        for subscript, replacement in cases:
            expected = bytearray(b"abcdef")
            expected[subscript] = replacement
            value = make(b"abcdef")
            value[subscript] = replacement
            assert bytes(value) == bytes(expected)

    @pytest.mark.parametrize("make", RAW[:2])
    @pytest.mark.parametrize("subscript", [slice(None), slice(None, None, -1)])
    def test_raw_self_slice_assignment_uses_snapshot(self, make, subscript):
        expected = bytearray(b"abcdef")
        expected[subscript] = expected
        value = make(b"abcdef")
        value[subscript] = value
        assert bytes(value) == bytes(expected)

    @pytest.mark.parametrize("make", RAW[:2])
    def test_raw_slice_assignment_errors_preserve_state(self, make):
        cases = [
            (slice(4, 1, -2), b"X", ValueError),
            (slice(4, 1, 2), b"X", ValueError),
            (slice(None, None, 0), b"", ValueError),
            (10**100, 0, IndexError),
            (object(), 0, TypeError),
            (slice(None), object(), TypeError),
        ]
        for subscript, replacement, error in cases:
            value = make(b"abcdef")
            with pytest.raises(error):
                value[subscript] = replacement
            assert bytes(value) == b"abcdef"

    @pytest.mark.parametrize("make", RAW[:2])
    def test_raw_scalar_assignment_matches_bytearray_index_protocol(self, make):
        class Index:
            def __init__(self, value):
                self.value = value

            def __index__(self):
                return self.value

        class InvalidIndex:
            def __index__(self):
                return "not an integer"

        class RaisingIndex:
            def __init__(self, error_type, message):
                self.error_type = error_type
                self.message = message

            def __index__(self):
                raise self.error_type(self.message)

        for replacement in (0, True, Index(255)):
            native = bytearray(b"abc")
            native[1] = replacement
            value = make(b"abc")
            value[1] = replacement
            assert bytes(value) == bytes(native)

        for replacement, error_type, message in [
            (-1, ValueError, None),
            (256, ValueError, None),
            (10**100, ValueError, None),
            (Index(10**100), ValueError, None),
            (object(), TypeError, None),
            (InvalidIndex(), TypeError, None),
            (RaisingIndex(RuntimeError, "runtime-scalar"), RuntimeError, "runtime-scalar"),
            (RaisingIndex(TypeError, "type-scalar"), TypeError, "type-scalar"),
            (RaisingIndex(OverflowError, "overflow-scalar"), OverflowError, "overflow-scalar"),
        ]:
            native = bytearray(b"abc")
            with pytest.raises(error_type) as native_error:
                native[1] = replacement
            value = make(b"abc")
            with pytest.raises(error_type) as wrapper_error:
                value[1] = replacement
            if message is not None:
                assert str(wrapper_error.value) == str(native_error.value) == message
            assert bytes(value) == b"abc"

    @pytest.mark.parametrize("make", RAW[:2])
    def test_raw_slice_assignment_matches_bytearray_rhs_protocol(self, make):
        cases = [
            (slice(1, 3), lambda: b"XY"),
            (slice(1, 3), lambda: bytearray(b"XY")),
            (slice(1, 3), lambda: memoryview(b"XY")),
            (slice(1, 3), lambda: [88, 89]),
            (slice(1, 3), lambda: range(88, 90)),
            (slice(1, 3), lambda: iter([88, 89])),
            (slice(4, 1, -2), lambda: (item for item in (88, 89))),
            (slice(4, 1, 2), lambda: iter(())),
        ]

        for subscript, replacement in cases:
            native = bytearray(b"abcdef")
            native[subscript] = replacement()
            value = make(b"abcdef")
            value[subscript] = replacement()
            assert bytes(value) == bytes(native)

    @pytest.mark.parametrize("make", RAW[:2])
    def test_raw_slice_assignment_conversion_and_length_errors_are_atomic(self, make):
        class RaisingIterator:
            def __iter__(self):
                yield 88
                raise RuntimeError("iterator-boom")

        class RaisingElement:
            def __index__(self):
                raise RuntimeError("element-boom")

        cases = [
            (slice(1, 3), lambda: "XY", TypeError, None),
            (slice(1, 3), lambda: [88, 256], ValueError, None),
            (slice(1, 3), lambda: [88, RaisingElement()], RuntimeError, "element-boom"),
            (slice(1, 3), RaisingIterator, RuntimeError, "iterator-boom"),
        ]

        for subscript, replacement, error_type, message in cases:
            native = bytearray(b"abcdef")
            with pytest.raises(error_type) as native_error:
                native[subscript] = replacement()
            value = make(b"abcdef")
            with pytest.raises(error_type) as wrapper_error:
                value[subscript] = replacement()
            if message is not None:
                assert str(wrapper_error.value) == str(native_error.value) == message
            assert bytes(value) == b"abcdef"

        for subscript, replacement in [
            (slice(1, 3), b"X"),
            (slice(4, 1, -2), b"X"),
            (slice(4, 1, 2), b"X"),
        ]:
            value = make(b"abcdef")
            with pytest.raises(ValueError):
                value[subscript] = replacement
            assert bytes(value) == b"abcdef"

    @pytest.mark.parametrize("make", UTF8)
    def test_utf8_length_index_and_iteration_match_str(self, make):
        native = "aé€🦀e\u0301z"
        value = make(native)
        assert len(value) == len(native)
        assert list(value) == list(native)
        assert [value[index] for index in range(-len(native), len(native))] == [
            native[index] for index in range(-len(native), len(native))
        ]
        assert value.remaining() == len(native.encode("utf-8"))

    @pytest.mark.parametrize("make", RAW)
    def test_raw_contains_matches_native_bytes(self, make):
        native = b"abc"
        for item in [97, 256, -1, b"bc", bytearray(b"bc"), memoryview(b"bc"), "a", object()]:
            value = make(native)
            try:
                expected = item in native
            except Exception as error:
                with pytest.raises(type(error)):
                    _ = item in value
            else:
                assert (item in value) == expected

    @pytest.mark.parametrize("make", UTF8)
    def test_utf8_contains_matches_native_str(self, make):
        native = "aé€🦀e\u0301z"
        for item in ["é€", "", 97, -1, "é".encode(), bytearray(b"x"), memoryview(b"x"), object()]:
            value = make(native)
            try:
                expected = item in native
            except Exception as error:
                with pytest.raises(type(error)):
                    _ = item in value
            else:
                assert (item in value) == expected

    def test_raw_comparisons_follow_native_bytes_protocol(self):
        for make in self.RAW:
            value = make(b"abc")
            assert value == b"abc"
            assert b"abc" == value
            assert value == bytearray(b"abc")
            assert value == memoryview(b"abc")
            assert value < b"abd"
            with pytest.raises(TypeError):
                _ = value < "abc"

    @pytest.mark.parametrize("make", RAW)
    @pytest.mark.parametrize(
        "comparison",
        [operator.lt, operator.le, operator.eq, operator.ne, operator.gt, operator.ge],
    )
    def test_raw_comparisons_match_exact_bytearray_both_directions(self, make, comparison):
        native = b"abc"
        value = make(native)
        for other in (bytearray(native), bytearray(b"abd")):
            assert comparison(value, other) == comparison(native, other)
            assert comparison(other, value) == comparison(other, native)

    @pytest.mark.parametrize("make", RAW)
    @pytest.mark.parametrize("comparison", [operator.eq, operator.ne])
    def test_raw_eq_ne_match_exact_memoryview_both_directions(self, make, comparison):
        native = b"abc"
        value = make(native)
        for other in (memoryview(native), memoryview(b"abd")):
            assert comparison(value, other) == comparison(native, other)
            assert comparison(other, value) == comparison(other, native)

    @pytest.mark.parametrize("make", RAW)
    @pytest.mark.parametrize("comparison", [operator.lt, operator.le, operator.gt, operator.ge])
    def test_raw_ordering_with_exact_memoryview_raises_both_directions(self, make, comparison):
        value = make(b"abc")
        other = memoryview(b"abc")
        with pytest.raises(TypeError):
            comparison(value, other)
        with pytest.raises(TypeError):
            comparison(other, value)

    @pytest.mark.parametrize("make", RAW)
    def test_raw_multidimensional_memoryview_is_unequal(self, make):
        value = make(b"abcd")
        other = memoryview(b"abcd").cast("B", shape=[2, 2])
        assert value != other
        assert other != value

    @pytest.mark.parametrize("make", RAW)
    def test_bytearray_subclass_uses_one_call_fallback(self, make):
        sentinel = object()

        class ComparisonOverride(bytearray):
            def __init__(self, value):
                super().__init__(value)
                self.calls = 0

            def __eq__(self, other):
                self.calls += 1
                return sentinel

        other = ComparisonOverride(b"abc")
        value = make(b"abc")
        assert value.__eq__(other) is NotImplemented
        assert other.calls == 0
        assert (value == other) is sentinel
        assert other.calls == 1

    @pytest.mark.parametrize("left", RAW)
    @pytest.mark.parametrize("right", RAW)
    @pytest.mark.parametrize(
        "comparison",
        [operator.lt, operator.le, operator.eq, operator.ne, operator.gt, operator.ge],
    )
    def test_raw_wrapper_comparisons_match_bytes_across_strategies(self, left, right, comparison):
        for left_value, right_value in [(b"abc", b"abc"), (b"abc", b"abd")]:
            assert comparison(left(left_value), right(right_value)) == comparison(left_value, right_value)

    @pytest.mark.parametrize("left", UTF8)
    @pytest.mark.parametrize("right", UTF8)
    @pytest.mark.parametrize(
        "comparison",
        [operator.lt, operator.le, operator.eq, operator.ne, operator.gt, operator.ge],
    )
    def test_utf8_wrapper_comparisons_match_str_across_strategies(self, left, right, comparison):
        for left_value, right_value in [("abc", "abc"), ("abc", "abd")]:
            assert comparison(left(left_value), right(right_value)) == comparison(left_value, right_value)

    @pytest.mark.parametrize("make", RAW + UTF8)
    def test_comparisons_preserve_non_bool_results(self, make):
        sentinel = object()

        class SentinelComparison:
            def __eq__(self, other):
                return sentinel

            def __ne__(self, other):
                return sentinel

            def __lt__(self, other):
                return sentinel

            def __le__(self, other):
                return sentinel

            def __gt__(self, other):
                return sentinel

            def __ge__(self, other):
                return sentinel

        value = make(b"abc") if make in self.RAW else make("abc")
        other = SentinelComparison()
        assert (value == other) is sentinel
        assert (value != other) is sentinel
        assert (value < other) is sentinel
        assert (value <= other) is sentinel
        assert (value > other) is sentinel
        assert (value >= other) is sentinel

    @pytest.mark.parametrize("make", RAW + UTF8)
    @pytest.mark.parametrize(
        "comparison",
        [operator.lt, operator.le, operator.eq, operator.ne, operator.gt, operator.ge],
        ids=["lt", "le", "eq", "ne", "gt", "ge"],
    )
    def test_custom_left_comparisons_match_native_without_retry(self, make, comparison):
        sentinel = object()

        class CustomLeft:
            def __init__(self):
                self.calls = 0

            def _compare(self, other):
                self.calls += 1
                return NotImplemented if self.calls == 1 else sentinel

            __lt__ = _compare
            __le__ = _compare
            __eq__ = _compare
            __ne__ = _compare
            __gt__ = _compare
            __ge__ = _compare

        native = b"abc" if make in self.RAW else "abc"
        native_left = CustomLeft()
        try:
            expected = comparison(native_left, native)
        except TypeError:
            wrapper_left = CustomLeft()
            with pytest.raises(TypeError):
                comparison(wrapper_left, make(native))
        else:
            wrapper_left = CustomLeft()
            assert comparison(wrapper_left, make(native)) is expected

        assert native_left.calls == 1
        assert wrapper_left.calls == 1

    @pytest.mark.parametrize("make", RAW + UTF8)
    @pytest.mark.parametrize("method", ["get_uint", "get_uint_le", "get_int", "get_int_le"])
    def test_variable_width_validates_before_reading(self, make, method):
        value = make(b"\x01" if make in self.RAW else "a")
        before = bytes(value) if hasattr(value, "__bytes__") else str(value)
        for width, error in [(-1, ValueError), (9, ValueError), (10**100, ValueError), ("1", TypeError)]:
            with pytest.raises(error):
                getattr(value, method)(width)
            after = bytes(value) if hasattr(value, "__bytes__") else str(value)
            assert after == before

    @pytest.mark.parametrize("make", RAW + UTF8)
    @pytest.mark.parametrize("method", ["get_uint", "get_uint_le", "get_int", "get_int_le"])
    def test_variable_width_edges_match_buffer_contract(self, make, method):
        is_raw = make in self.RAW
        payloads = [b"", b"\x01", b"\x01" * 8]
        if make not in (self.RAW[0], self.UTF8[0]):
            payloads.append(b"\x01" * 63)

        for payload in payloads:
            source = payload if is_raw else payload.decode("ascii")
            for width in (0, 1, 8):
                value = make(source)
                before = bytes(value) if hasattr(value, "__bytes__") else str(value)
                if width <= len(payload):
                    getattr(value, method)(width)
                    if width == 0:
                        after = bytes(value) if hasattr(value, "__bytes__") else str(value)
                        assert after == before
                else:
                    with pytest.raises(BufferError):
                        getattr(value, method)(width)
                    after = bytes(value) if hasattr(value, "__bytes__") else str(value)
                    assert after == before

    @pytest.mark.parametrize("make", [lambda: Utf8Buffer.from_str("éx"), lambda: Utf8BytesMut.from_str("éx")])
    def test_utf8_truncate_uses_char_offsets(self, make):
        value = make()
        value.truncate(1)  # keep one Unicode scalar value
        assert str(value) == "é"
        value = make()
        value.truncate(2)  # length in characters: no-op
        assert str(value) == "éx"
        value.truncate(99)  # beyond the character count: no-op
        assert str(value) == "éx"

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
    def test_utf8_split_out_of_range_raises(self, make_buf):
        # "café" has four characters; a char offset beyond that is out of range
        buf = make_buf()
        with pytest.raises(ValueError):
            buf.split_to(5)

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
        assert len(buf) == 4  # Unicode scalar values; remaining() is still five bytes.
        assert buf.remaining() == 5

    def test_crab_emoji_len(self):
        buf = Utf8BytesMut.from_str("\U0001f980")
        assert len(buf) == 1  # One Unicode scalar value; remaining() is still four bytes.
        assert buf.remaining() == 4

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


# ---------------------------------------------------------------------------
# UTF-8 char-offset semantics for truncate / split / slice / byte_len
# ---------------------------------------------------------------------------


class TestUtf8CharOffsets:
    """Offset-taking string methods interpret offsets as Unicode scalar values."""

    def test_len_counts_scalar_values(self):
        # c, a, f, é, space, crab emoji -> six scalar values
        assert len(Utf8BytesMut.from_str("café 🦀")) == 6

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("café"),
            lambda: Utf8BytesMut.from_str("café"),
        ],
        ids=["Utf8Buffer", "Utf8BytesMut"],
    )
    def test_truncate_at_char_offset_crossing_multibyte(self, make_buf):
        buf = make_buf()
        buf.truncate(3)  # keep "caf"; the é (byte offsets 3..5) is dropped
        assert str(buf) == "caf"

    def test_truncate_between_multibyte_chars(self):
        buf = Utf8BytesMut.from_str("éé")
        buf.truncate(1)  # keep one scalar value across a multibyte boundary
        assert str(buf) == "é"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Bytes.from_str("café 🦀"),
            lambda: CompactUtf8Bytes.from_str("café 🦀"),
        ],
        ids=["Utf8Bytes", "CompactUtf8Bytes"],
    )
    def test_split_to_at_char_offset(self, make_buf):
        head = make_buf().split_to(4)  # four scalar values -> "café"
        assert str(head) == "café"

    @pytest.mark.parametrize(
        "make_buf",
        [
            lambda: Utf8Buffer.from_str("café 🦀"),
            lambda: Utf8Bytes.from_str("café 🦀"),
            lambda: CompactUtf8Bytes.from_str("café 🦀"),
            lambda: Utf8BytesMut.from_str("café 🦀"),
        ],
        ids=["Utf8Buffer", "Utf8Bytes", "CompactUtf8Bytes", "Utf8BytesMut"],
    )
    def test_byte_len_reports_byte_length(self, make_buf):
        buf = make_buf()
        assert len(buf) == 6
        assert buf.byte_len() == len("café 🦀".encode("utf-8"))
