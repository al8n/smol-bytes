use super::*;

/// A strategy for the [`SmolBytes`] type, storing bytes inline where possible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Inline(());

impl Strategy for SmolBytes<Inline> {
  fn slice(&self, range: impl RangeBounds<usize>) -> Self {
    let len = self.len();

    let begin = match range.start_bound() {
      Bound::Included(&n) => n,
      Bound::Excluded(&n) => n.checked_add(1).expect("out of range"),
      Bound::Unbounded => 0,
    };

    let end = match range.end_bound() {
      Bound::Included(&n) => n.checked_add(1).expect("out of range"),
      Bound::Excluded(&n) => n,
      Bound::Unbounded => len,
    };

    let Some(slen) = end.checked_sub(begin) else {
      panic!(
        "range start must not be greater than end: {:?} <= {:?}",
        begin, end,
      );
    };

    assert!(
      end <= len,
      "range end out of bounds: {:?} <= {:?}",
      end,
      len,
    );

    if slen <= INLINE_CAP {
      let mut new_buf = [0u8; INLINE_CAP];
      new_buf[..slen].copy_from_slice(&self.as_slice()[begin..end]);
      return Self::inline(new_buf, 0, slen);
    }

    match &self.repr {
      Repr::Inline { .. } => {
        unreachable!("slice length exceeds inline capacity");
      }
      Repr::Heap(bytes) => Self::heap(bytes.slice(begin..end)),
    }
  }

  fn split_to(&mut self, at: usize) -> Self {
    let len = self.len();
    if at == len {
      return mem::take(self);
    }

    if at == 0 {
      return Self::new();
    }

    assert!(at <= len, "split_to out of bounds: {:?} <= {:?}", at, len,);

    // first, check if output can be inline
    let ret = if at <= INLINE_CAP {
      let mut buf = [0u8; INLINE_CAP];
      let src = self.as_slice();
      buf[..at].copy_from_slice(&src[..at]);
      Self::inline(buf, 0, at)
    } else {
      // output cannot be inline, which means it must be heap allocated
      let mut bytes = self.repr.unwrap_heap_mut().clone();
      bytes.truncate(at);
      Self::heap(bytes)
    };

    // second, check if self can be made inline
    let remaining_size = len - at;
    if remaining_size <= INLINE_CAP {
      // check if we already are inline, if so, adjust cur, avoid copy
      if let Repr::Inline { cur, len, .. } = &mut self.repr {
        *cur = len.to_u8() - (remaining_size as u8);
        return ret;
      }

      let mut buf = [0u8; INLINE_CAP];
      let src = self.as_slice();
      buf[..remaining_size].copy_from_slice(&src[at..len]);
      let _ = mem::replace(
        &mut self.repr,
        Repr::Inline {
          len: unsafe { InlineSize::from_u8(remaining_size as u8) },
          buf,
          cur: 0,
        },
      );
    } else {
      // self remains heap allocated
      self.repr.unwrap_heap_mut().advance(at);
    }
    ret
  }

  fn split_off(&mut self, at: usize) -> Self {
    let len = self.len();
    if at == len {
      return Self::new();
    }

    if at == 0 {
      return mem::take(self);
    }

    assert!(at <= len, "split_off out of bounds: {:?} <= {:?}", at, len,);

    // first, check if output would be inline
    let output_size = len - at;
    let ret = if output_size <= INLINE_CAP {
      let mut buf = [0u8; INLINE_CAP];
      let src = self.as_slice();
      buf[..output_size].copy_from_slice(&src[at..len]);
      Self::inline(buf, 0, output_size)
    } else {
      // output cannot be inline, which means it must be heap allocated
      let mut bytes = self.repr.unwrap_heap_mut().clone();
      bytes.advance(at);
      Self::heap(bytes)
    };

    // second, check if self can be made inline
    if at <= INLINE_CAP {
      // check if we already are inline, if so, adjust len
      if let Repr::Inline { len, cur, .. } = &mut self.repr {
        // len represents the end position in buf, so we need cur + at
        *len = unsafe { InlineSize::from_u8((*cur as usize + at) as u8) };
        return ret;
      }

      let mut buf = [0u8; INLINE_CAP];
      let src = self.as_slice();
      buf[..at].copy_from_slice(&src[..at]);
      let _ = mem::replace(
        &mut self.repr,
        Repr::Inline {
          len: unsafe { InlineSize::from_u8(at as u8) },
          buf,
          cur: 0,
        },
      );
    } else {
      // self remains heap allocated
      self.repr.unwrap_heap_mut().truncate(at);
    }

    ret
  }

  fn truncate(&mut self, new_len: usize) {
    match &mut self.repr {
      Repr::Inline { len, .. } => {
        if new_len <= len.to_usize() {
          *len = unsafe { InlineSize::from_u8(new_len as u8) };
        }
      }
      Repr::Heap(bytes) => {
        if new_len <= INLINE_CAP {
          let mut buf = [0u8; INLINE_CAP];
          buf[..new_len].copy_from_slice(&bytes[..new_len]);
          let _ = mem::replace(
            &mut self.repr,
            Repr::Inline {
              len: unsafe { InlineSize::from_u8(new_len as u8) },
              buf,
              cur: 0,
            },
          );
        } else {
          bytes.truncate(new_len);
        }
      }
    }
  }

  fn advance(&mut self, cnt: usize) {
    match &mut self.repr {
      Repr::Inline { len, cur, .. } => {
        let remaining = len.to_u8() - *cur;
        assert!(
          cnt <= remaining as usize,
          "cannot advance past `remaining`: {:?} <= {:?}",
          cnt,
          remaining,
        );
        *cur += cnt as u8;
      }
      Repr::Heap(bytes) => {
        // check if we can make inline after advance
        let len = bytes.len();
        assert!(
          cnt <= len,
          "cannot advance past `remaining`: {:?} <= {:?}",
          cnt,
          len,
        );

        let remaining = len - cnt;
        if remaining <= INLINE_CAP {
          let mut buf = [0u8; INLINE_CAP];
          let src = &bytes[cnt..];
          buf[..remaining].copy_from_slice(src);
          let _ = mem::replace(&mut self.repr, Repr::inline(buf, 0, remaining));
        } else {
          bytes.advance(cnt);
        }
      }
    }
  }

  fn copy_to_bytes(&mut self, len: usize) -> bytes::Bytes {
    self.split_to(len).into()
  }

  fn clear(&mut self) {
    let _ = mem::replace(
      &mut self.repr,
      Repr::Inline {
        len: InlineSize::_V0,
        buf: [0; INLINE_CAP],
        cur: 0,
      },
    );
  }
}
