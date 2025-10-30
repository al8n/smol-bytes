//! Const I/O methods for Buffer matching bytes::Buf and bytes::BufMut trait signatures.
//!
//! These methods allow Buffer to be used in no_std/no_alloc environments while providing
//! the same API as the bytes crate.

use super::{Buffer, InlineSize, TryGetError, TryPutError, panic_advance, panic_does_not_fit};

// https://en.wikipedia.org/wiki/Sign_extension
const fn sign_extend(val: u64, nbytes: usize) -> i64 {
  let shift = (8 - nbytes) * 8;
  (val << shift) as i64 >> shift
}

macro_rules! impl_try_get {
  ($this:ident, $typ:tt::$conv:tt) => {{
    const SIZE: usize = core::mem::size_of::<$typ>();
    
    let remaining = $this.remaining();

    if remaining < SIZE {
      return Err(TryGetError {
        requested: SIZE,
        available: remaining,
      });
    }

    unsafe {
      let ptr = $this.buf.as_ptr().add($this.cur.to_usize());
      let val = $typ::$conv(*(ptr as *const _ as *const [_; SIZE]));

      $this.cur = InlineSize::from_u8($this.cur.to_u8() + SIZE as u8);
      Ok(val)
    }
  }};
  (le($this:ident, $typ:tt, $len_to_read:expr)) => {{
      const SIZE: usize = core::mem::size_of::<$typ>();

      let remaining = $this.remaining();
      if remaining < $len_to_read {
        return Err(TryGetError {
          requested: $len_to_read,
          available: remaining,
        });
      }

      if $len_to_read > SIZE {
        panic_does_not_fit(SIZE, $len_to_read);
      }

      let mut buf = [0u8; SIZE];

      unsafe {
        // SAFETY: checked above that there is enough remaining data
        // For LE: copy to the START of the buffer [data..., 0, 0, 0]
        core::ptr::copy_nonoverlapping(
          $this.buf.as_ptr().add($this.cur.to_usize()).cast::<u8>(),
          buf.as_mut_ptr(),
          $len_to_read,
        );
        let val = $typ::from_le_bytes(buf);

        $this.cur = InlineSize::from_u8($this.cur.to_u8() + $len_to_read as u8);
        Ok(val)
      }
  }};
  (be($this:ident, $typ:tt, $len_to_read:expr)) => {{
      const SIZE: usize = core::mem::size_of::<$typ>();

      let remaining = $this.remaining();
      if remaining < $len_to_read {
        return Err(TryGetError {
          requested: $len_to_read,
          available: remaining,
        });
      }

      if $len_to_read > SIZE {
        panic_does_not_fit(SIZE, $len_to_read);
      }

      let slice_at = SIZE - $len_to_read;
      let mut buf = [0u8; SIZE];

      unsafe {
        // SAFETY: checked above that there is enough remaining data
        // For BE: copy to the END of the buffer [0, 0, 0, ..., data...]
        core::ptr::copy_nonoverlapping(
          $this.buf.as_ptr().add($this.cur.to_usize()).cast::<u8>(),
          buf.as_mut_ptr().add(slice_at),
          $len_to_read,
        );
        let val = $typ::from_be_bytes(buf);

        $this.cur = InlineSize::from_u8($this.cur.to_u8() + $len_to_read as u8);
        Ok(val)
      }
  }};
}

macro_rules! try_get_and_get {
  ($long:literal: { $($typ:ident::$conv:ident$(::$order:ident)?),+$(,)? }) => {
    paste::paste! {
      $(
        #[doc = "Gets a `" $typ "` from `self` in " $long " byte order."]
        ///
        /// The current position is advanced by the size of the type.
        ///
        /// # Panics
        ///
        /// This function panics if there is not enough remaining data in `self`.
        #[inline]
        pub fn [<get_ $typ:lower $(_ $order)?>](&mut self) -> $typ {
          self.[<try_get_ $typ:lower $(_ $order)?>]().unwrap_or_else(|e| panic_advance(e.available, e.requested))
        }

        #[doc = "Tries to get a `" $typ "` from `self` in " $long " byte order."]
        ///
        /// The current position is advanced by the size of the type.
        ///
        /// # Errors
        ///
        /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
        #[inline]
        pub const fn [<try_get_ $typ:lower $(_ $order)?>](&mut self) -> Result<$typ, TryGetError> {
          impl_try_get!(self, $typ::$conv)
        }
      )*
    }
  };
}

macro_rules! try_put_and_put {
  ($long:literal: { $($typ:ident::$conv:ident$(::$order:ident)?),+$(,)? }) => {
    paste::paste! {
      $(
        #[doc = "Puts a `" $typ "` to `self` in " $long " byte order."]
        ///
        /// The current length is advanced by the size of the type.
        ///
        /// # Panics
        ///
        /// This function panics if there is not enough remaining capacity in `self`.
        #[inline]
        pub fn [<put_ $typ:lower $(_ $order)?>](&mut self, val: $typ) {
          self.[<try_put_ $typ:lower $(_ $order)?>](val)
            .unwrap_or_else(|e| panic_advance(e.available, e.requested))
        }

        #[doc = "Tries to put a `" $typ "` to `self` in " $long " byte order."]
        ///
        /// The current length is advanced by the size of the type.
        ///
        /// # Errors
        ///
        /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
        #[inline]
        pub const fn [<try_put_ $typ:lower $(_ $order)?>](&mut self, val: $typ) -> Result<(), TryPutError> {
          self.try_put_slice(val.$conv().as_slice())
        }
      )*
    }
  };
}

impl Buffer {
  try_get_and_get!(
    "little-endian": {
      u16::from_le_bytes::le,
      i16::from_le_bytes::le,
      u32::from_le_bytes::le,
      i32::from_le_bytes::le,
      u64::from_le_bytes::le,
      i64::from_le_bytes::le,
      u128::from_le_bytes::le,
      i128::from_le_bytes::le,
    }
  );

  try_get_and_get!(
    "big-endian": {
      u16::from_be_bytes,
      i16::from_be_bytes,
      u32::from_be_bytes,
      i32::from_be_bytes,
      u64::from_be_bytes,
      i64::from_be_bytes,
      u128::from_be_bytes,
      i128::from_be_bytes,
    }
  );

  try_get_and_get!(
    "native-endian": {
      u16::from_ne_bytes::ne,
      i16::from_ne_bytes::ne,
      u32::from_ne_bytes::ne,
      i32::from_ne_bytes::ne,
      u64::from_ne_bytes::ne,
      i64::from_ne_bytes::ne,
      u128::from_ne_bytes::ne,
      i128::from_ne_bytes::ne,
    }
  );

  /// Gets an unsigned 8 bit integer from `self`.
  ///
  /// The current position is advanced by 1.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Buffer;
  ///
  /// let mut buf = Buffer::try_from(&b"\x08 hello"[..]).unwrap();
  /// assert_eq!(8, buf.get_u8());
  /// assert_eq!(5, buf.remaining());
  /// ```
  ///
  /// # Panics
  ///
  /// This function panics if there is no more remaining data in `self`.
  #[inline]
  pub fn get_u8(&mut self) -> u8 {
    match self.try_get_u8() {
      Ok(v) => v,
      Err(e) => panic_advance(e.available, e.requested),
    }
  }

  /// Tries to get an unsigned 8 bit integer from `self`.
  ///
  /// The current position is advanced by 1.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  #[inline]
  pub const fn try_get_u8(&mut self) -> Result<u8, TryGetError> {
    if self.remaining() < 1 {
      return Err(TryGetError {
        requested: 1,
        available: self.remaining(),
      });
    }

    // SAFETY: checked above that there is at least 1 byte remaining
    let ret = unsafe { self.buf[self.cur.to_usize()].assume_init_read() };
    self.cur = unsafe { InlineSize::from_u8(self.cur.to_u8() + 1) };
    Ok(ret)
  }

  /// Gets a signed 8 bit integer from `self`.
  ///
  /// The current position is advanced by 1.
  ///
  /// # Panics
  ///
  /// This function panics if there is no more remaining data in `self`.
  #[inline]
  pub fn get_i8(&mut self) -> i8 {
    Self::get_u8(self) as i8
  }

  /// Tries to get a signed 8 bit integer from `self`.
  ///
  /// The current position is advanced by 1.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  #[inline]
  pub const fn try_get_i8(&mut self) -> Result<i8, TryGetError> {
    match Self::try_get_u8(self) {
      Ok(v) => Ok(v as i8),
      Err(e) => Err(e),
    }
  }

  /// Gets an IEEE754 single-precision (4 bytes) floating point number from
  /// `self` in big-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Buffer;
  ///
  /// let mut buf = Buffer::try_from(&[0x41, 0x48, 0x00, 0x00][..]).unwrap();
  /// assert_eq!(12.5, buf.get_f32());
  /// ```
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`.
  #[inline]
  pub fn get_f32(&mut self) -> f32 {
    f32::from_bits(self.get_u32())
  }

  /// Tries to get an IEEE754 single-precision (4 bytes) floating point number from
  /// `self` in big-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  #[inline]
  pub const fn try_get_f32(&mut self) -> Result<f32, TryGetError> {
    match self.try_get_u32() {
      Ok(bits) => Ok(f32::from_bits(bits)),
      Err(e) => Err(e),
    }
  }

  /// Gets an IEEE754 single-precision (4 bytes) floating point number from
  /// `self` in little-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`.
  #[inline]
  pub fn get_f32_le(&mut self) -> f32 {
    f32::from_bits(self.get_u32_le())
  }

  /// Tries to get an IEEE754 single-precision (4 bytes) floating point number from
  /// `self` in little-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  #[inline]
  pub const fn try_get_f32_le(&mut self) -> Result<f32, TryGetError> {
    match self.try_get_u32_le() {
      Ok(bits) => Ok(f32::from_bits(bits)),
      Err(e) => Err(e),
    }
  }

  /// Gets an IEEE754 single-precision (4 bytes) floating point number from
  /// `self` in native-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`.
  #[inline]
  pub fn get_f32_ne(&mut self) -> f32 {
    f32::from_bits(self.get_u32_ne())
  }

  /// Tries to get an IEEE754 single-precision (4 bytes) floating point number from
  /// `self` in native-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  #[inline]
  pub const fn try_get_f32_ne(&mut self) -> Result<f32, TryGetError> {
    match self.try_get_u32_ne() {
      Ok(bits) => Ok(f32::from_bits(bits)),
      Err(e) => Err(e),
    }
  }

  /// Gets an IEEE754 double-precision (8 bytes) floating point number from
  /// `self` in big-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`.
  #[inline]
  pub fn get_f64(&mut self) -> f64 {
    f64::from_bits(self.get_u64())
  }

  /// Tries to get an IEEE754 double-precision (8 bytes) floating point number from
  /// `self` in big-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  #[inline]
  pub const fn try_get_f64(&mut self) -> Result<f64, TryGetError> {
    match self.try_get_u64() {
      Ok(bits) => Ok(f64::from_bits(bits)),
      Err(e) => Err(e),
    }
  }

  /// Gets an IEEE754 double-precision (8 bytes) floating point number from
  /// `self` in little-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`.
  #[inline]
  pub fn get_f64_le(&mut self) -> f64 {
    f64::from_bits(self.get_u64_le())
  }

  /// Tries to get an IEEE754 double-precision (8 bytes) floating point number from
  /// `self` in little-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  #[inline]
  pub const fn try_get_f64_le(&mut self) -> Result<f64, TryGetError> {
    match self.try_get_u64_le() {
      Ok(bits) => Ok(f64::from_bits(bits)),
      Err(e) => Err(e),
    }
  }

  /// Gets an IEEE754 double-precision (8 bytes) floating point number from
  /// `self` in native-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`.
  #[inline]
  pub fn get_f64_ne(&mut self) -> f64 {
    f64::from_bits(self.get_u64_ne())
  }

  /// Tries to get an IEEE754 double-precision (8 bytes) floating point number from
  /// `self` in native-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  #[inline]
  pub const fn try_get_f64_ne(&mut self) -> Result<f64, TryGetError> {
    match self.try_get_u64_ne() {
      Ok(bits) => Ok(f64::from_bits(bits)),
      Err(e) => Err(e),
    }
  }

  /// Gets an unsigned n-byte integer from `self` in big-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Buffer;
  ///
  /// let mut buf = Buffer::try_from(&b"\x01\x02\x03"[..]).unwrap();
  /// assert_eq!(0x010203, buf.get_uint(3));
  /// ```
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn get_uint(&mut self, nbytes: usize) -> u64 {
    self.try_get_uint(nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Tries to get an unsigned n-byte integer from `self` in big-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub fn try_get_uint(&mut self, nbytes: usize) -> Result<u64, TryGetError> {
    impl_try_get!(be(self, u64, nbytes))
  }

  /// Gets an unsigned n-byte integer from `self` in little-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn get_uint_le(&mut self, nbytes: usize) -> u64 {
    self.try_get_uint_le(nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Tries to get an unsigned n-byte integer from `self` in little-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub fn try_get_uint_le(&mut self, nbytes: usize) -> Result<u64, TryGetError> {
    impl_try_get!(le(self, u64, nbytes))
  }

  /// Gets an unsigned n-byte integer from `self` in native-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn get_uint_ne(&mut self, nbytes: usize) -> u64 {
    self.try_get_uint_ne(nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Tries to get an unsigned n-byte integer from `self` in native-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub fn try_get_uint_ne(&mut self, nbytes: usize) -> Result<u64, TryGetError> {
    if cfg!(target_endian = "big") {
      self.try_get_uint(nbytes)
    } else {
      self.try_get_uint_le(nbytes)
    }
  }

  /// Gets a signed n-byte integer from `self` in big-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn get_int(&mut self, nbytes: usize) -> i64 {
    sign_extend(self.get_uint(nbytes), nbytes)
  }

  /// Tries to get a signed n-byte integer from `self` in big-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub fn try_get_int(&mut self, nbytes: usize) -> Result<i64, TryGetError> {
    impl_try_get!(be(self, i64, nbytes))
  }

  /// Gets a signed n-byte integer from `self` in little-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn get_int_le(&mut self, nbytes: usize) -> i64 {
    sign_extend(self.get_uint_le(nbytes), nbytes)
  }

  /// Tries to get a signed n-byte integer from `self` in little-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub fn try_get_int_le(&mut self, nbytes: usize) -> Result<i64, TryGetError> {
    impl_try_get!(le(self, i64, nbytes))
  }

  /// Gets a signed n-byte integer from `self` in native-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining data in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn get_int_ne(&mut self, nbytes: usize) -> i64 {
    self.try_get_int_ne(nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Tries to get a signed n-byte integer from `self` in native-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryGetError)` if there is not enough remaining data in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub fn try_get_int_ne(&mut self, nbytes: usize) -> Result<i64, TryGetError> {
    if cfg!(target_endian = "big") {
      self.try_get_int(nbytes)
    } else {
      self.try_get_int_le(nbytes)
    }
  }
}

impl Buffer {
  try_put_and_put!(
    "little-endian": {
      u16::to_le_bytes::le,
      i16::to_le_bytes::le,
      u32::to_le_bytes::le,
      i32::to_le_bytes::le,
      u64::to_le_bytes::le,
      i64::to_le_bytes::le,
      u128::to_le_bytes::le,
      i128::to_le_bytes::le,
    }
  );

  try_put_and_put!(
    "big-endian": {
      u16::to_be_bytes,
      i16::to_be_bytes,
      u32::to_be_bytes,
      i32::to_be_bytes,
      u64::to_be_bytes,
      i64::to_be_bytes,
      u128::to_be_bytes,
      i128::to_be_bytes,
    }
  );

  try_put_and_put!(
    "native-endian": {
      u16::to_ne_bytes::ne,
      i16::to_ne_bytes::ne,
      u32::to_ne_bytes::ne,
      i32::to_ne_bytes::ne,
      u64::to_ne_bytes::ne,
      i64::to_ne_bytes::ne,
      u128::to_ne_bytes::ne,
      i128::to_ne_bytes::ne,
    }
  );

  /// Writes a signed 8 bit integer to `self`.
  ///
  /// The current position is advanced by 1.
  ///
  /// # Examples
  ///
  /// ```
  /// use smol_bytes::Buffer;
  ///
  /// let mut buf = Buffer::new();
  /// buf.put_i8(-5);
  /// assert_eq!(buf[0], 251);
  /// ```
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`.
  #[inline]
  pub fn put_i8(&mut self, n: i8) {
    self.try_put_i8(n).unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Tries to write a signed 8 bit integer to `self`.
  ///
  /// The current position is advanced by 1.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  #[inline]
  pub const fn try_put_i8(&mut self, n: i8) -> Result<(), TryPutError> {
    self.try_put_u8(n as u8)
  }

  /// Writes an unsigned n-byte integer to `self` in big-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn put_uint(&mut self, n: u64, nbytes: usize) {
    self.try_put_uint(n, nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested));
  }

  /// Tries to write an unsigned n-byte integer to `self` in big-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub const fn try_put_uint(&mut self, n: u64, nbytes: usize) -> Result<(), TryPutError> {
    if nbytes > 8 {
      panic!("nbytes must be <= 8");
    }
    let available = self.remaining_mut();
    if available < nbytes {
      return Err(TryPutError {
        requested: nbytes,
        available,
      });
    }

    let bytes = n.to_be_bytes();
    let mut i = 0;
    while i < nbytes {
      unsafe {
        *self
          .buf
          .as_mut_ptr()
          .add(self.len.to_usize() + i)
          .cast::<u8>() = bytes[8 - nbytes + i];
      }
      i += 1;
    }
    self.len = unsafe { InlineSize::from_u8(self.len.to_u8() + nbytes as u8) };
    Ok(())
  }

  /// Writes an unsigned n-byte integer to `self` in little-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn put_uint_le(&mut self, n: u64, nbytes: usize) {
    self.try_put_uint_le(n, nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Tries to write an unsigned n-byte integer to `self` in little-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub const fn try_put_uint_le(&mut self, n: u64, nbytes: usize) -> Result<(), TryPutError> {
    if nbytes > 8 {
      panic!("nbytes must be <= 8");
    }
    let available = self.remaining_mut();
    if available < nbytes {
      return Err(TryPutError {
        requested: nbytes,
        available,
      });
    }

    let bytes = n.to_le_bytes();
    let mut i = 0;
    while i < nbytes {
      unsafe {
        *self
          .buf
          .as_mut_ptr()
          .add(self.len.to_usize() + i)
          .cast::<u8>() = bytes[i];
      }
      i += 1;
    }
    self.len = unsafe { InlineSize::from_u8(self.len.to_u8() + nbytes as u8) };
    Ok(())
  }

  /// Writes an unsigned n-byte integer to `self` in native-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn put_uint_ne(&mut self, n: u64, nbytes: usize) {
    self.try_put_uint_ne(n, nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested))
  }

  /// Tries to write an unsigned n-byte integer to `self` in native-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub const fn try_put_uint_ne(&mut self, n: u64, nbytes: usize) -> Result<(), TryPutError> {
    if cfg!(target_endian = "big") {
      self.try_put_uint(n, nbytes)
    } else {
      self.try_put_uint_le(n, nbytes)
    }
  }

  /// Writes a signed n-byte integer to `self` in big-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn put_int(&mut self, n: i64, nbytes: usize) {
    self.try_put_int(n, nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested));
  }

  /// Tries to write a signed n-byte integer to `self` in big-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub const fn try_put_int(&mut self, n: i64, nbytes: usize) -> Result<(), TryPutError> {
    self.try_put_uint(n as u64, nbytes)
  }

  /// Writes a signed n-byte integer to `self` in little-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn put_int_le(&mut self, n: i64, nbytes: usize) {
    self.try_put_int_le(n, nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested));
  }

  /// Tries to write a signed n-byte integer to `self` in little-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub const fn try_put_int_le(&mut self, n: i64, nbytes: usize) -> Result<(), TryPutError> {
    self.try_put_uint_le(n as u64, nbytes)
  }

  /// Writes a signed n-byte integer to `self` in native-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`,
  /// or if `nbytes` is greater than 8.
  #[inline]
  pub fn put_int_ne(&mut self, n: i64, nbytes: usize) {
    self.try_put_int_ne(n, nbytes).unwrap_or_else(|e| panic_advance(e.available, e.requested));
  }

  /// Tries to write a signed n-byte integer to `self` in native-endian byte order.
  ///
  /// The current position is advanced by `nbytes`.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  ///
  /// # Panics
  ///
  /// This function panics if `nbytes` is greater than 8.
  #[inline]
  pub const fn try_put_int_ne(&mut self, n: i64, nbytes: usize) -> Result<(), TryPutError> {
    if cfg!(target_endian = "big") {
      self.try_put_int(n, nbytes)
    } else {
      self.try_put_int_le(n, nbytes)
    }
  }

  /// Writes an IEEE754 single-precision (4 bytes) floating point number to
  /// `self` in big-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`.
  #[inline]
  pub fn put_f32(&mut self, n: f32) {
    self.put_u32(n.to_bits())
  }

  /// Tries to write an IEEE754 single-precision (4 bytes) floating point number to
  /// `self` in big-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  #[inline]
  pub const fn try_put_f32(&mut self, n: f32) -> Result<(), TryPutError> {
    self.try_put_u32(n.to_bits())
  }

  /// Writes an IEEE754 single-precision (4 bytes) floating point number to
  /// `self` in little-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`.
  #[inline]
  pub fn put_f32_le(&mut self, n: f32) {
    self.put_u32_le(n.to_bits())
  }

  /// Tries to write an IEEE754 single-precision (4 bytes) floating point number to
  /// `self` in little-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  #[inline]
  pub const fn try_put_f32_le(&mut self, n: f32) -> Result<(), TryPutError> {
    self.try_put_u32_le(n.to_bits())
  }

  /// Writes an IEEE754 single-precision (4 bytes) floating point number to
  /// `self` in native-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`.
  #[inline]
  pub fn put_f32_ne(&mut self, n: f32) {
    self.put_u32_ne(n.to_bits())
  }

  /// Tries to write an IEEE754 single-precision (4 bytes) floating point number to
  /// `self` in native-endian byte order.
  ///
  /// The current position is advanced by 4.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  #[inline]
  pub const fn try_put_f32_ne(&mut self, n: f32) -> Result<(), TryPutError> {
    self.try_put_u32_ne(n.to_bits())
  }

  /// Writes an IEEE754 double-precision (8 bytes) floating point number to
  /// `self` in big-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`.
  #[inline]
  pub fn put_f64(&mut self, n: f64) {
    self.put_u64(n.to_bits())
  }

  /// Tries to write an IEEE754 double-precision (8 bytes) floating point number to
  /// `self` in big-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  #[inline]
  pub const fn try_put_f64(&mut self, n: f64) -> Result<(), TryPutError> {
    self.try_put_u64(n.to_bits())
  }

  /// Writes an IEEE754 double-precision (8 bytes) floating point number to
  /// `self` in little-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`.
  #[inline]
  pub fn put_f64_le(&mut self, n: f64) {
    self.put_u64_le(n.to_bits())
  }

  /// Tries to write an IEEE754 double-precision (8 bytes) floating point number to
  /// `self` in little-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  #[inline]
  pub const fn try_put_f64_le(&mut self, n: f64) -> Result<(), TryPutError> {
    self.try_put_u64_le(n.to_bits())
  }

  /// Writes an IEEE754 double-precision (8 bytes) floating point number to
  /// `self` in native-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Panics
  ///
  /// This function panics if there is not enough remaining capacity in `self`.
  #[inline]
  pub fn put_f64_ne(&mut self, n: f64) {
    self.put_u64_ne(n.to_bits())
  }

  /// Tries to write an IEEE754 double-precision (8 bytes) floating point number to
  /// `self` in native-endian byte order.
  ///
  /// The current position is advanced by 8.
  ///
  /// # Errors
  ///
  /// Returns `Err(TryPutError)` if there is not enough remaining capacity in `self`.
  #[inline]
  pub const fn try_put_f64_ne(&mut self, n: f64) -> Result<(), TryPutError> {
    self.try_put_u64_ne(n.to_bits())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_u8() {
    let mut buf = Buffer::try_from(&[0x01, 0x02, 0x03][..]).unwrap();
    assert_eq!(buf.get_u8(), 0x01);
    assert_eq!(buf.get_u8(), 0x02);
    assert_eq!(buf.get_u8(), 0x03);
    assert_eq!(buf.remaining(), 0);
  }

  #[test]
  fn test_try_get_u8_error() {
    let mut buf = Buffer::new();
    assert!(buf.try_get_u8().is_err());
  }

  #[test]
  fn test_get_i8() {
    let mut buf = Buffer::try_from(&[0xFF, 0x7F, 0x80][..]).unwrap();
    assert_eq!(buf.get_i8(), -1);
    assert_eq!(buf.get_i8(), 127);
    assert_eq!(buf.get_i8(), -128);
  }

  #[test]
  fn test_get_u16() {
    let mut buf = Buffer::try_from(&[0x01, 0x02, 0x03, 0x04][..]).unwrap();
    assert_eq!(buf.get_u16(), 0x0102);
    assert_eq!(buf.get_u16(), 0x0304);
  }

  #[test]
  fn test_get_u16_le() {
    let mut buf = Buffer::try_from(&[0x01, 0x02, 0x03, 0x04][..]).unwrap();
    assert_eq!(buf.get_u16_le(), 0x0201);
    assert_eq!(buf.get_u16_le(), 0x0403);
  }

  #[test]
  fn test_get_i16() {
    let mut buf = Buffer::try_from(&[0xFF, 0xFF, 0x7F, 0xFF][..]).unwrap();
    assert_eq!(buf.get_i16(), -1);
    assert_eq!(buf.get_i16(), 32767);
  }

  #[test]
  fn test_get_u32() {
    let mut buf = Buffer::try_from(&[0x01, 0x02, 0x03, 0x04][..]).unwrap();
    assert_eq!(buf.get_u32(), 0x01020304);
  }

  #[test]
  fn test_get_u32_le() {
    let mut buf = Buffer::try_from(&[0x01, 0x02, 0x03, 0x04][..]).unwrap();
    assert_eq!(buf.get_u32_le(), 0x04030201);
  }

  #[test]
  fn test_get_i32() {
    let mut buf = Buffer::try_from(&[0xFF, 0xFF, 0xFF, 0xFF][..]).unwrap();
    assert_eq!(buf.get_i32(), -1);
  }

  #[test]
  fn test_get_u64() {
    let mut buf = Buffer::try_from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]).unwrap();
    assert_eq!(buf.get_u64(), 0x0102030405060708);
  }

  #[test]
  fn test_get_u64_le() {
    let mut buf = Buffer::try_from(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08][..]).unwrap();
    assert_eq!(buf.get_u64_le(), 0x0807060504030201);
  }

  #[test]
  fn test_get_i64() {
    let mut buf = Buffer::try_from(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF][..]).unwrap();
    assert_eq!(buf.get_i64(), -1);
  }

  #[test]
  fn test_get_u128() {
    let mut buf = Buffer::try_from(&[
      0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
      0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10
    ][..]).unwrap();
    assert_eq!(buf.get_u128(), 0x0102030405060708090A0B0C0D0E0F10);
  }

  #[test]
  fn test_get_u128_le() {
    let mut buf = Buffer::try_from(&[
      0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
      0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10
    ][..]).unwrap();
    assert_eq!(buf.get_u128_le(), 0x100F0E0D0C0B0A090807060504030201);
  }

  #[test]
  fn test_get_i128() {
    let mut buf = Buffer::try_from(&[
      0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
      0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
    ][..]).unwrap();
    assert_eq!(buf.get_i128(), -1);
  }

  #[test]
  fn test_get_f32() {
    let mut buf = Buffer::try_from(&[0x41, 0x48, 0x00, 0x00][..]).unwrap();
    assert_eq!(buf.get_f32(), 12.5);
  }

  #[test]
  fn test_get_f32_le() {
    let mut buf = Buffer::try_from(&[0x00, 0x00, 0x48, 0x41][..]).unwrap();
    assert_eq!(buf.get_f32_le(), 12.5);
  }

  #[test]
  fn test_get_f64() {
    let mut buf = Buffer::try_from(&[0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00][..]).unwrap();
    assert_eq!(buf.get_f64(), 12.5);
  }

  #[test]
  fn test_get_f64_le() {
    let mut buf = Buffer::try_from(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40][..]).unwrap();
    assert_eq!(buf.get_f64_le(), 12.5);
  }

  #[test]
  fn test_get_uint() {
    let mut buf = Buffer::try_from(&[0x01, 0x02, 0x03][..]).unwrap();
    assert_eq!(buf.get_uint(3), 0x010203);
    assert_eq!(buf.remaining(), 0);
  }

  #[test]
  fn test_get_uint_le() {
    let mut buf = Buffer::try_from(&[0x01, 0x02, 0x03][..]).unwrap();
    assert_eq!(buf.get_uint_le(3), 0x030201);
    assert_eq!(buf.remaining(), 0);
  }

  #[test]
  fn test_get_int() {
    let mut buf = Buffer::try_from(&[0xFF, 0xFF, 0xFF][..]).unwrap();
    assert_eq!(buf.get_int(3), -1);
  }

  #[test]
  fn test_get_int_le() {
    let mut buf = Buffer::try_from(&[0xFF, 0xFF, 0xFF][..]).unwrap();
    assert_eq!(buf.get_int_le(3), -1);
  }

  // Put tests
  #[test]
  fn test_put_u8() {
    let mut buf = Buffer::new();
    buf.put_u8(0x01);
    buf.put_u8(0x02);
    assert_eq!(&buf[..], &[0x01, 0x02]);
  }

  #[test]
  fn test_try_put_u8_error() {
    let mut buf = Buffer::try_from(&[0u8; 62][..]).unwrap();
    assert!(buf.try_put_u8(0xFF).is_err());
  }

  #[test]
  fn test_put_i8() {
    let mut buf = Buffer::new();
    buf.put_i8(-1);
    buf.put_i8(127);
    assert_eq!(&buf[..], &[0xFF, 0x7F]);
  }

  #[test]
  fn test_put_u16() {
    let mut buf = Buffer::new();
    buf.put_u16(0x0102);
    assert_eq!(&buf[..], &[0x01, 0x02]);
  }

  #[test]
  fn test_put_u16_le() {
    let mut buf = Buffer::new();
    buf.put_u16_le(0x0102);
    assert_eq!(&buf[..], &[0x02, 0x01]);
  }

  #[test]
  fn test_put_i16() {
    let mut buf = Buffer::new();
    buf.put_i16(-1);
    assert_eq!(&buf[..], &[0xFF, 0xFF]);
  }

  #[test]
  fn test_put_u32() {
    let mut buf = Buffer::new();
    buf.put_u32(0x01020304);
    assert_eq!(&buf[..], &[0x01, 0x02, 0x03, 0x04]);
  }

  #[test]
  fn test_put_u32_le() {
    let mut buf = Buffer::new();
    buf.put_u32_le(0x01020304);
    assert_eq!(&buf[..], &[0x04, 0x03, 0x02, 0x01]);
  }

  #[test]
  fn test_put_i32() {
    let mut buf = Buffer::new();
    buf.put_i32(-1);
    assert_eq!(&buf[..], &[0xFF, 0xFF, 0xFF, 0xFF]);
  }

  #[test]
  fn test_put_u64() {
    let mut buf = Buffer::new();
    buf.put_u64(0x0102030405060708);
    assert_eq!(&buf[..], &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
  }

  #[test]
  fn test_put_u64_le() {
    let mut buf = Buffer::new();
    buf.put_u64_le(0x0102030405060708);
    assert_eq!(&buf[..], &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
  }

  #[test]
  fn test_put_i64() {
    let mut buf = Buffer::new();
    buf.put_i64(-1);
    assert_eq!(&buf[..], &[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
  }

  #[test]
  fn test_put_u128() {
    let mut buf = Buffer::new();
    buf.put_u128(0x0102030405060708090A0B0C0D0E0F10);
    assert_eq!(&buf[..], &[
      0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
      0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10
    ]);
  }

  #[test]
  fn test_put_u128_le() {
    let mut buf = Buffer::new();
    buf.put_u128_le(0x0102030405060708090A0B0C0D0E0F10);
    assert_eq!(&buf[..], &[
      0x10, 0x0F, 0x0E, 0x0D, 0x0C, 0x0B, 0x0A, 0x09,
      0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01
    ]);
  }

  #[test]
  fn test_put_i128() {
    let mut buf = Buffer::new();
    buf.put_i128(-1);
    assert_eq!(&buf[..], &[
      0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
      0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
    ]);
  }

  #[test]
  fn test_put_f32() {
    let mut buf = Buffer::new();
    buf.put_f32(12.5);
    assert_eq!(&buf[..], &[0x41, 0x48, 0x00, 0x00]);
  }

  #[test]
  fn test_put_f32_le() {
    let mut buf = Buffer::new();
    buf.put_f32_le(12.5);
    assert_eq!(&buf[..], &[0x00, 0x00, 0x48, 0x41]);
  }

  #[test]
  fn test_put_f64() {
    let mut buf = Buffer::new();
    buf.put_f64(12.5);
    assert_eq!(&buf[..], &[0x40, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
  }

  #[test]
  fn test_put_f64_le() {
    let mut buf = Buffer::new();
    buf.put_f64_le(12.5);
    assert_eq!(&buf[..], &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0x40]);
  }

  #[test]
  fn test_put_uint() {
    let mut buf = Buffer::new();
    buf.put_uint(0x010203, 3);
    assert_eq!(&buf[..], &[0x01, 0x02, 0x03]);
  }

  #[test]
  fn test_put_uint_le() {
    let mut buf = Buffer::new();
    buf.put_uint_le(0x010203, 3);
    assert_eq!(&buf[..], &[0x03, 0x02, 0x01]);
  }

  #[test]
  fn test_put_int() {
    let mut buf = Buffer::new();
    buf.put_int(-1, 3);
    assert_eq!(&buf[..], &[0xFF, 0xFF, 0xFF]);
  }

  #[test]
  fn test_put_int_le() {
    let mut buf = Buffer::new();
    buf.put_int_le(-1, 3);
    assert_eq!(&buf[..], &[0xFF, 0xFF, 0xFF]);
  }

  #[test]
  fn test_roundtrip_u16() {
    let mut buf = Buffer::new();
    let value: u16 = 0x1234;
    buf.put_u16(value);
    buf.advance(0);
    assert_eq!(buf.get_u16(), value);
  }

  #[test]
  fn test_roundtrip_u32() {
    let mut buf = Buffer::new();
    let value: u32 = 0x12345678;
    buf.put_u32(value);
    buf.advance(0);
    assert_eq!(buf.get_u32(), value);
  }

  #[test]
  fn test_roundtrip_u64() {
    let mut buf = Buffer::new();
    let value: u64 = 0x123456789ABCDEF0;
    buf.put_u64(value);
    buf.advance(0);
    assert_eq!(buf.get_u64(), value);
  }

  #[test]
  fn test_roundtrip_u128() {
    let mut buf = Buffer::new();
    let value: u128 = 0x123456789ABCDEF0FEDCBA9876543210;
    buf.put_u128(value);
    buf.advance(0);
    assert_eq!(buf.get_u128(), value);
  }

  #[test]
  fn test_roundtrip_f32() {
    let mut buf = Buffer::new();
    let value: f32 = core::f32::consts::PI;
    buf.put_f32(value);
    buf.advance(0);
    assert_eq!(buf.get_f32(), value);
  }

  #[test]
  fn test_roundtrip_f64() {
    let mut buf = Buffer::new();
    let value: f64 = core::f64::consts::PI;
    buf.put_f64(value);
    buf.advance(0);
    assert_eq!(buf.get_f64(), value);
  }

  #[test]
  fn test_mixed_operations() {
    let mut buf = Buffer::new();
    buf.put_u8(0x01);
    buf.put_u16(0x0203);
    buf.put_u32(0x04050607);
    buf.advance(0);

    assert_eq!(buf.get_u8(), 0x01);
    assert_eq!(buf.get_u16(), 0x0203);
    assert_eq!(buf.get_u32(), 0x04050607);
    assert_eq!(buf.remaining(), 0);
  }

  #[test]
  fn test_native_endian_consistency() {
    let mut buf = Buffer::new();
    let value: u32 = 0x12345678;
    buf.put_u32_ne(value);
    buf.advance(0);
    assert_eq!(buf.get_u32_ne(), value);
  }

  #[test]
  #[should_panic]
  fn test_get_u8_panic() {
    let mut buf = Buffer::new();
    buf.get_u8(); // Should panic - no data
  }

  #[test]
  #[should_panic]
  fn test_put_u8_panic() {
    let mut buf = Buffer::try_from(&[0u8; 62][..]).unwrap();
    buf.put_u8(0xFF); // Should panic - no capacity
  }

  #[test]
  fn test_try_methods_errors() {
    let mut empty_buf = Buffer::new();
    assert!(empty_buf.try_get_u8().is_err());
    assert!(empty_buf.try_get_i8().is_err());
    assert!(empty_buf.try_get_u16().is_err());
    assert!(empty_buf.try_get_i16().is_err());
    assert!(empty_buf.try_get_u32().is_err());
    assert!(empty_buf.try_get_i32().is_err());
    assert!(empty_buf.try_get_u64().is_err());
    assert!(empty_buf.try_get_i64().is_err());
    assert!(empty_buf.try_get_u128().is_err());
    assert!(empty_buf.try_get_i128().is_err());
    assert!(empty_buf.try_get_f32().is_err());
    assert!(empty_buf.try_get_f64().is_err());
    assert!(empty_buf.try_get_uint(3).is_err());
    assert!(empty_buf.try_get_int(3).is_err());

    let mut full_buf = Buffer::try_from(&[0u8; 62][..]).unwrap();
    assert!(full_buf.try_put_u8(0).is_err());
    assert!(full_buf.try_put_i8(0).is_err());
  }
}
