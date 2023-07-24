#![no_std]
use core::{ops, str};
use tinyvec::SliceVec;

pub struct SliceString<'a>(SliceVec<'a, u8>);

impl<'a> SliceString<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self(SliceVec::from_slice_len(buf, 0))
    }

    pub fn as_str(&self) -> &str {
        // split_at_unchecked?
        unsafe { str::from_utf8_unchecked(&self.0) }
    }

    pub fn as_mut_str(&mut self) -> &mut str {
        // split_at_unchecked?
        unsafe { str::from_utf8_unchecked_mut(&mut self.0) }
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn push_str(&mut self, string: &str) -> Result<(), ()> {
        let bytes = string.as_bytes();
        if self.capacity() < self.0.len() + bytes.len() {
            return Err(());
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }

    pub fn pop(&mut self) -> Option<char> {
        let ch = self.chars().rev().next()?;
        self.0.truncate(self.len() - ch.len_utf8());
        Some(ch)
    }

    pub fn push(&mut self, c: char) -> Result<(), ()> {
        let len = c.len_utf8();
        if self.capacity() < self.len() + len {
            return Err(());
        }
        if len == 1 {
            self.0.push(c as u8);
        } else {
            self.0
                .extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
        }
        Ok(())
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            assert!(self.is_char_boundary(new_len));
            self.0.truncate(new_len);
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
        assert_eq!(0, s.len());
        s.push_str("Hello world!").unwrap();
        assert_eq!(s.as_str(), "Hello world!");
    }
}
