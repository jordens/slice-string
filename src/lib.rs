#![no_std]
use core::{str, ops};

pub struct SliceString<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> SliceString<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self {buf, len: 0}
    }

    pub fn as_str(&self) -> &str {
        // split_at_unchecked?
        unsafe { str::from_utf8_unchecked(&self.buf[..self.len]) }
    }

    pub fn as_mut_str(&mut self) -> &mut str {
        // split_at_unchecked?
        unsafe { str::from_utf8_unchecked_mut(&mut self.buf[..self.len]) }
    }

    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn push_str(&mut self, string: &str) -> Result<(), ()> {
        let bytes = string.as_bytes();
        let len = self.len + bytes.len();
        if self.capacity() < len {
            return Err(());
        }
        self.buf[self.len..len].copy_from_slice(bytes);
        self.len = len;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<char> {
        let ch = self.chars().rev().next()?;
        self.len -= ch.len_utf8();
        Some(ch)
    }

    pub fn push(&mut self, c: char) -> Result<(), ()> {
        let len = c.len_utf8();
        if self.capacity() < self.len + len {
            return Err(())
        }
        if len == 1 {
            self.buf[self.len] = c as u8;
        } else {
            c.encode_utf8(&mut self
                    .buf[self.len..]);
        }
        self.len += len;
        Ok(())
    }

   pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len {
            assert!(self.is_char_boundary(new_len));
            self.len = new_len;
        }
    }
}

impl<'a> ops::Deref for SliceString<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let mut buf = [0u8; 32];
        let mut s = SliceString::new(&mut buf[..]);
        s.push_str("Hello world!").unwrap();
        assert_eq!(s.as_str(), "Hello world!");
    }
}
