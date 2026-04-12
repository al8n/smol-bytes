"""Shared strategy -- preserves heap allocations for fast bytes::Bytes interop."""

from .._smol_bytes.shared import Bytes, Utf8Bytes

__all__ = ["Bytes", "Utf8Bytes"]
