macro_rules! method_forward {
  ($repr:tt { $(
    $(#[$meta:meta])*
    fn $name:ident($( $arg:ident : $arg_ty:ty ),*) $(-> $ret:ty)?
  );+$(;)? }) => {
    $(
      $(#[$meta])*
      #[cfg_attr(not(coverage), inline(always))]
      fn $name(&mut self, $( $arg : $arg_ty ),*) $(-> $ret)? {
        match &mut self.$repr {
          Repr::Inline(buffer) => buffer.$name($( $arg ),*),
          Repr::Heap(bytes) => bytes.$name($( $arg ),*),
        }
      }
    )*
  };
  ($(
    $(#[$meta:meta])*
    fn $name:ident($($arg:ident : $arg_ty:ty ),*) $(-> $ret:ty)?
  );+$(;)?) => {
    $(
      $(#[$meta])*
      #[cfg_attr(not(coverage), inline(always))]
      fn $name(&mut self, $( $arg : $arg_ty ),*) $(-> $ret)? {
        Self::$name(self, $( $arg ),*)
      }
    )*
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

pub(super) use forward_buf_mut;
pub(super) use method_forward;

/// Formats arguments into the default shared [`Utf8Bytes`](crate::Utf8Bytes).
///
/// This is similar to `format!`, but returns a value that retains heap storage
/// when later shortened. Use `format_compact_utf8bytes!` when short results
/// should use inline storage.
///
/// # Examples
///
/// ```
/// use smol_bytes::{format_utf8bytes, Utf8Bytes};
///
/// let name = "Ada";
/// let greeting: Utf8Bytes = format_utf8bytes!("hello, {name}!");
/// assert_eq!(greeting.as_str(), "hello, Ada!");
/// ```
#[macro_export]
macro_rules! format_utf8bytes {
  ($($tt:tt)*) => {{
    let mut value = $crate::Utf8BytesMut::new();
    ::core::fmt::Write::write_fmt(&mut value, ::core::format_args!($($tt)*))
      .expect("a formatting trait implementation returned an error");
    value.freeze_shared()
  }};
}

/// Formats arguments into compact [`Utf8Bytes`](crate::compact::Utf8Bytes).
///
/// This is similar to `format!`, but returns a value that uses inline storage
/// when its contents fit, including after a larger value is later shortened.
/// Use `format_utf8bytes!` when retaining heap storage after shrinking is preferred.
///
/// # Examples
///
/// ```
/// use smol_bytes::{compact, format_compact_utf8bytes};
///
/// let name = "Ada";
/// let greeting: compact::Utf8Bytes = format_compact_utf8bytes!("hello, {name}!");
/// assert_eq!(greeting.as_str(), "hello, Ada!");
/// ```
#[macro_export]
macro_rules! format_compact_utf8bytes {
  ($($tt:tt)*) => {{
    let mut value = $crate::Utf8BytesMut::new();
    ::core::fmt::Write::write_fmt(&mut value, ::core::format_args!($($tt)*))
      .expect("a formatting trait implementation returned an error");
    value.freeze_compact()
  }};
}
