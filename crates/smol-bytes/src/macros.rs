macro_rules! method_forward {
  ($repr:tt { $(
    fn $name:ident($( $arg:ident : $arg_ty:ty ),*) $(-> $ret:ty)?
  );+$(;)? }) => {
    $(
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn $name(&mut self, $( $arg : $arg_ty ),*) $(-> $ret)? {
        match &mut self.$repr {
          Repr::Inline(buffer) => buffer.$name($( $arg ),*),
          Repr::Heap(bytes) => bytes.$name($( $arg ),*),
        }
      }
    )*
  };
  ($(
    fn $name:ident($($arg:ident : $arg_ty:ty ),*) $(-> $ret:ty)?
  );+$(;)?) => {
    $(
      #[cfg_attr(not(tarpaulin), inline(always))]
      fn $name(&mut self, $( $arg : $arg_ty ),*) $(-> $ret)? {
        Self::$name(self, $( $arg ),*)
      }
    )*
  };
}

macro_rules! forward_buf {
  ($repr:tt { $($suffix:ident), +$(,)? }) => {
    paste::paste! {
      $(
        $crate::macros::method_forward! { $repr {
          fn [< get_ $suffix >]() -> $suffix;
          fn [< get_ $suffix _le >]() -> $suffix;
          fn [< get_ $suffix _ne >]() -> $suffix;
          fn [< try_get_ $suffix >]() -> Result<$suffix, $crate::TryGetError>;
          fn [< try_get_ $suffix _le >]() -> Result<$suffix, $crate::TryGetError>;
          fn [< try_get_ $suffix _ne >]() -> Result<$suffix, $crate::TryGetError>;
        }}
      )*

      $crate::macros::method_forward! {$repr {
        fn get_i8() -> i8;
        fn get_u8() -> u8;
        fn try_get_i8() -> Result<i8, $crate::TryGetError>;
        fn try_get_u8() -> Result<u8, $crate::TryGetError>;

        fn get_uint(nbytes: usize) -> u64;
        fn get_uint_le(nbytes: usize) -> u64;
        fn get_uint_ne(nbytes: usize) -> u64;
        fn get_int(nbytes: usize) -> i64;
        fn get_int_le(nbytes: usize) -> i64;
        fn get_int_ne(nbytes: usize) -> i64;

        fn try_get_uint(nbytes: usize) -> Result<u64, $crate::TryGetError>;
        fn try_get_uint_le(nbytes: usize) -> Result<u64, $crate::TryGetError>;
        fn try_get_uint_ne(nbytes: usize) -> Result<u64, $crate::TryGetError>;
        fn try_get_int(nbytes: usize) -> Result<i64, $crate::TryGetError>;
        fn try_get_int_le(nbytes: usize) -> Result<i64, $crate::TryGetError>;
        fn try_get_int_ne(nbytes: usize) -> Result<i64, $crate::TryGetError>;
      }}
    }
  };
  ($($suffix:ident), +$(,)?) => {
    paste::paste! {
      $(
        $crate::macros::method_forward! {
          fn [< get_ $suffix >]() -> $suffix;
          fn [< get_ $suffix _le >]() -> $suffix;
          fn [< get_ $suffix _ne >]() -> $suffix;
          fn [< try_get_ $suffix >]() -> Result<$suffix, $crate::TryGetError>;
          fn [< try_get_ $suffix _le >]() -> Result<$suffix, $crate::TryGetError>;
          fn [< try_get_ $suffix _ne >]() -> Result<$suffix, $crate::TryGetError>;
        }
      )*

      $crate::macros::method_forward! {
        fn get_i8() -> i8;
        fn get_u8() -> u8;
        fn try_get_i8() -> Result<i8, $crate::TryGetError>;
        fn try_get_u8() -> Result<u8, $crate::TryGetError>;

        fn get_uint(nbytes: usize) -> u64;
        fn get_uint_le(nbytes: usize) -> u64;
        fn get_uint_ne(nbytes: usize) -> u64;
        fn get_int(nbytes: usize) -> i64;
        fn get_int_le(nbytes: usize) -> i64;
        fn get_int_ne(nbytes: usize) -> i64;

        fn try_get_uint(nbytes: usize) -> Result<u64, $crate::TryGetError>;
        fn try_get_uint_le(nbytes: usize) -> Result<u64, $crate::TryGetError>;
        fn try_get_uint_ne(nbytes: usize) -> Result<u64, $crate::TryGetError>;
        fn try_get_int(nbytes: usize) -> Result<i64, $crate::TryGetError>;
        fn try_get_int_le(nbytes: usize) -> Result<i64, $crate::TryGetError>;
        fn try_get_int_ne(nbytes: usize) -> Result<i64, $crate::TryGetError>;
      }
    }
  };
}

macro_rules! forward_buf_mut {
  ($repr:tt { $($suffix:ident), +$(,)? }) => {
    paste::paste! {
      $(
        $crate::macros::method_forward! { $repr {
          fn [< put_ $suffix >](val: $suffix);
          fn [< put_ $suffix _le >](val: $suffix);
          fn [< put_ $suffix _ne >](val: $suffix);
        }}
      )*

      $crate::macros::method_forward! {$repr {
        fn put_i8(val: i8);
        fn put_u8(val: u8);

        fn put_uint(n: u64, nbytes: usize);
        fn put_uint_le(n: u64, nbytes: usize);
        fn put_uint_ne(n: u64, nbytes: usize);
        fn put_int(n: i64, nbytes: usize);
        fn put_int_le(n: i64, nbytes: usize);
        fn put_int_ne(n: i64, nbytes: usize);
      }}
    }
  };
  ($($suffix:ident), +$(,)?) => {
    paste::paste! {
      $(
        $crate::macros::method_forward! {
          fn [< put_ $suffix >](val: $suffix);
          fn [< put_ $suffix _le >](val: $suffix);
          fn [< put_ $suffix _ne >](val: $suffix);
        }
      )*

      $crate::macros::method_forward! {
        fn put_i8(val: i8);
        fn put_u8(val: u8);

        fn put_uint(n: u64, nbytes: usize);
        fn put_uint_le(n: u64, nbytes: usize);
        fn put_uint_ne(n: u64, nbytes: usize);

        fn put_int(n: i64, nbytes: usize);
        fn put_int_le(n: i64, nbytes: usize);
        fn put_int_ne(n: i64, nbytes: usize);
      }
    }
  };
}

pub(super) use forward_buf;
pub(super) use forward_buf_mut;
pub(super) use method_forward;
