"""Compact strategy -- aggressively inlines to minimize memory usage."""

from .._smol_bytes.compact import Bytes, Utf8Bytes

__all__ = ["Bytes", "Utf8Bytes"]
