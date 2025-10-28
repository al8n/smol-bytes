use core::{
  mem::{transmute, MaybeUninit},
  ptr::{copy_nonoverlapping, write_bytes},
  slice::from_raw_parts_mut,
};

use bytes::{buf::UninitSlice, BufMut, TryGetError};

/// Number of bytes that can be stored inline.
pub const INLINE_CAP: usize = InlineSize::MAX as usize;

/// A type used internally to encode inline lengths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum InlineSize {
  _V0 = 0,
  _V1,
  _V2,
  _V3,
  _V4,
  _V5,
  _V6,
  _V7,
  _V8,
  _V9,
  _V10,
  _V11,
  _V12,
  _V13,
  _V14,
  _V15,
  _V16,
  _V17,
  _V18,
  _V19,
  _V20,
  _V21,
  _V22,
  _V23,
  _V24,
  _V25,
  _V26,
  _V27,
  _V28,
  _V29,
  _V30,
  _V31,
  _V32,
  _V33,
  _V34,
  _V35,
  _V36,
  _V37,
  _V38,
  _V39,
  _V40,
  _V41,
  _V42,
  _V43,
  _V44,
  _V45,
  _V46,
  _V47,
  _V48,
  _V49,
  _V50,
  _V51,
  _V52,
  _V53,
  _V54,
  _V55,
  _V56,
  _V57,
  _V58,
  _V59,
  _V60,
  _V61,
  _V62,
}

impl core::ops::Sub for InlineSize {
  type Output = Self;

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn sub(self, rhs: Self) -> Self::Output {
    // Safety: subtraction result is guaranteed to be less than or equal to INLINE_CAP
    unsafe { InlineSize::from_u8(self.to_u8() - rhs.to_u8()) }
  }
}

impl InlineSize {
  const MAX: u8 = InlineSize::_V62 as u8;

  /// # Safety
  ///
  /// `value` must be less than or equal to [`INLINE_CAP`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const unsafe fn from_u8(value: u8) -> Self {
    debug_assert!(value <= InlineSize::MAX);
    transmute::<u8, InlineSize>(value)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn to_u8(self) -> u8 {
    self as u8
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub(crate) const fn to_usize(self) -> usize {
    self as usize
  }
}

#[derive(Clone, Copy)]
pub(crate) struct InlineStorage {
  // The write cursor
  len: InlineSize,
  // The read cursor
  cur: InlineSize,
  buf: [MaybeUninit<u8>; INLINE_CAP],
}

impl core::ops::Deref for InlineStorage {
  type Target = [u8];

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref(&self) -> &Self::Target {
    self.as_slice()
  }
}

impl core::ops::DerefMut for InlineStorage {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.as_mut_slice()
  }
}

impl InlineStorage {
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn new() -> Self {
    Self {
      len: InlineSize::_V0,
      cur: InlineSize::_V0,
      buf: [const { MaybeUninit::uninit() }; INLINE_CAP],
    }
  }

  /// Safety:
  /// - `len` must be less than or equal to [`INLINE_CAP`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const unsafe fn zeroed(len: usize) -> Self {
    let mut storage = [const { MaybeUninit::uninit() }; INLINE_CAP];
    core::ptr::write_bytes(storage.as_mut_ptr(), 0, len);
    Self {
      cur: InlineSize::_V0,
      // SAFETY: len is guaranteed to be less than or equal to INLINE_CAP
      len: unsafe { InlineSize::from_u8(len as u8) },
      buf: storage,
    }
  }

  /// Safety:
  /// - `len` must be less than or equal to [`INLINE_CAP`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  #[allow(unused)]
  pub const unsafe fn from_array(buf: [u8; INLINE_CAP], len: usize) -> Self {
    Self {
      cur: InlineSize::_V0,
      // SAFETY: len is guaranteed to be less than or equal to INLINE_CAP
      len: unsafe { InlineSize::from_u8(len as u8) },
      // SAFETY: all bytes are initialized
      buf: unsafe { transmute::<[u8; INLINE_CAP], [MaybeUninit<u8>; INLINE_CAP]>(buf) },
    }
  }

  /// # Safety
  /// - the length of `src` must be less than or equal to [`INLINE_CAP`].
  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const unsafe fn copy_from_slice(src: &[u8]) -> Self {
    let len = src.len();
    let mut storage = [const { MaybeUninit::uninit() }; INLINE_CAP];

    // SAFETY: caller guarantees that `len` is less than or equal to `INLINE_CAP`.
    copy_nonoverlapping(src.as_ptr(), storage.as_mut_ptr() as _, len);

    Self {
      // SAFETY: caller guarantees that `len` is less than or equal to `INLINE_CAP`.
      len: InlineSize::from_u8(len as u8),
      cur: InlineSize::_V0,
      buf: storage,
    }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn remaining(&self) -> usize {
    self.len.to_usize() - self.cur as usize
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn len(&self) -> usize {
    self.len.to_usize()
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn remaining_mut(&self) -> usize {
    INLINE_CAP - (self.len.to_usize())
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn set_len(&mut self, len: usize) {
    debug_assert!(len <= INLINE_CAP, "set_len out of bounds");
    self.len = unsafe { InlineSize::from_u8(len as u8) };
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub fn advance(&mut self, requested: usize) {
    let available = (self.len - self.cur).to_usize();
    if available < requested {
      panic_advance(&TryGetError {
        requested,
        available,
      })
    }
    self.cur = unsafe { InlineSize::from_u8(self.cur.to_u8() + requested as u8) };
  }

  pub fn truncate(&mut self, new_len: usize) {
    if new_len == 0 {
      return self.clear();
    }
    let remaining = self.remaining();

    if new_len >= remaining {
      return;
    }

    let cur = self.cur as usize;
    self.buf.copy_within(cur..cur + new_len, 0);
    self.len = unsafe { InlineSize::from_u8(new_len as u8) };
    self.cur = InlineSize::_V0;
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn clear(&mut self) {
    self.len = InlineSize::_V0;
    self.cur = InlineSize::_V0;
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<u8>] {
    let len = self.len.to_usize();
    unsafe { from_raw_parts_mut(self.buf.as_mut_ptr().add(len), INLINE_CAP - len) }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn append_slice(&mut self, src: &[u8]) -> bool {
    let remaining = Self::remaining_mut(self);
    let len = src.len();

    if len > remaining {
      return false;
    }

    let slen = self.len.to_usize();
    unsafe {
      copy_nonoverlapping(src.as_ptr(), self.buf.as_mut_ptr().add(slen) as _, len);
    }
    self.len = unsafe { InlineSize::from_u8(slen as u8 + len as u8) };
    true
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_slice(&self) -> &[u8] {
    let ptr = self.buf.as_ptr() as *const u8;
    let remaining = self.remaining();
    unsafe { core::slice::from_raw_parts(ptr.add(self.cur.to_usize()), remaining) }
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  pub const fn as_mut_slice(&mut self) -> &mut [u8] {
    let ptr = self.buf.as_mut_ptr() as *mut u8;
    let remaining = self.remaining();
    unsafe { core::slice::from_raw_parts_mut(ptr.add(self.cur.to_usize()), remaining) }
  }
}

impl core::fmt::Debug for InlineStorage {
  // The code is copied from the bytes crate's Debug implementation.
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let slice = self.as_slice();
    write!(f, "b\"")?;
    for &b in slice {
      // https://doc.rust-lang.org/reference/tokens.html#byte-escapes
      if b == b'\n' {
        write!(f, "\\n")?;
      } else if b == b'\r' {
        write!(f, "\\r")?;
      } else if b == b'\t' {
        write!(f, "\\t")?;
      } else if b == b'\\' || b == b'"' {
        write!(f, "\\{}", b as char)?;
      } else if b == b'\0' {
        write!(f, "\\0")?;
      // ASCII printable
      } else if (0x20..0x7f).contains(&b) {
        write!(f, "{}", b as char)?;
      } else {
        write!(f, "\\x{:02x}", b)?;
      }
    }
    write!(f, "\"")?;
    Ok(())
  }
}

unsafe impl BufMut for InlineStorage {
  #[cfg_attr(not(tarpaulin), inline(always))]
  fn remaining_mut(&self) -> usize {
    Self::remaining_mut(self)
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  unsafe fn advance_mut(&mut self, cnt: usize) {
    let remaining = self.remaining_mut();
    if cnt > remaining {
      panic_advance(&TryGetError {
        requested: cnt,
        available: remaining,
      })
    }

    self.len = unsafe { InlineSize::from_u8(self.len.to_u8() + cnt as u8) };
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn chunk_mut(&mut self) -> &mut UninitSlice {
    let len = self.len.to_usize();
    if len >= INLINE_CAP {
      return UninitSlice::new(&mut []);
    }
    UninitSlice::uninit(&mut self.buf[len..])
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn put_slice(&mut self, src: &[u8]) {
    let available = self.remaining_mut();
    let requested = src.len();

    if requested > available {
      panic_advance(&TryGetError {
        requested,
        available,
      });
    }

    let slen = self.len.to_usize();
    unsafe {
      copy_nonoverlapping(
        src.as_ptr(),
        self.buf.as_mut_ptr().add(slen) as _,
        requested,
      );
    }
    self.len = unsafe { InlineSize::from_u8(slen as u8 + requested as u8) };
  }

  #[cfg_attr(not(tarpaulin), inline(always))]
  fn put_bytes(&mut self, val: u8, requested: usize) {
    let available = self.remaining_mut();
    if available < requested {
      panic_advance(&TryGetError {
        requested,
        available,
      });
    }

    // Safety: we have already checked that there is enough capacity.
    unsafe {
      write_bytes(
        self.buf.as_mut_ptr().add(self.len.to_usize()),
        val,
        requested,
      );
    }
  }
}

/// Panic with a nice error message.
#[cold]
fn panic_advance(error_info: &TryGetError) -> ! {
  panic!(
    "advance out of bounds: the len is {} but advancing by {}",
    error_info.available, error_info.requested
  );
}
