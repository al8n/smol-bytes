use super::Buffer;

impl core::fmt::Debug for Buffer {
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

impl core::fmt::LowerHex for Buffer {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    for &b in self.as_slice() {
      write!(f, "{:02x}", b)?;
    }
    Ok(())
  }
}

impl core::fmt::UpperHex for Buffer {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    for &b in self.as_slice() {
      write!(f, "{:02X}", b)?;
    }
    Ok(())
  }
}

impl core::fmt::Write for Buffer {
  #[cfg_attr(not(coverage), inline(always))]
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    if self.remaining_mut() >= s.len() {
      self.put_slice(s.as_bytes());
      Ok(())
    } else {
      Err(core::fmt::Error)
    }
  }
}
