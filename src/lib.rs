#![cfg_attr(not(test), no_std)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(missing_docs)]
#![deny(warnings)]

//! This module implements support for a String-like structure that is backed by a slice.

#[cfg(feature = "ufmt-impl")]
mod ufmt;

use core::{fmt, hash, ops, str};
pub use tinyvec;
use tinyvec::SliceVec; // re-export

/// A UTF-8-encoded growable string backed by a `u8` slice.
///
/// This supports some of the API from `std::String` and dereferences
/// to `core::str`.
#[repr(transparent)]
#[derive(Default)]
pub struct SliceString<'a>(SliceVec<'a, u8>);

impl<'a> SliceString<'a> {
    /// Create a new empty `SliceString` from a mutable slice.
    ///
    /// The capacity of the string will be the slice length.
    pub fn new(buf: &'a mut [u8]) -> Self {
        // Length-0 slices are always valid UTF-8.
        unsafe { Self::from_utf8_unchecked(buf, 0) }
    }

    /// Create a new `SliceString` from a [SliceVec].
    ///
    /// # Safety
    /// The data in `SliceVec` must be valid UTF-8.
    pub unsafe fn new_unchecked(buf: SliceVec<'a, u8>) -> Self {
        Self(buf)
    }

    /// Create a new `SliceString` from a mutable slice.
    ///
    /// The data in `buf[..len]` are checked to contain valid UTF-8.
    pub fn from_utf8(buf: &'a mut [u8], len: usize) -> Result<Self, str::Utf8Error> {
        str::from_utf8(&buf[..len])?;
        // UTF-8 validity of the buffer up to `len` has just been checked.
        Ok(unsafe { Self::from_utf8_unchecked(buf, len) })
    }

    /// Create a new `SliceString` from a mutable slice.
    ///
    /// # Safety
    /// The data in `buf[..len]` must be valid UTF-8.
    pub unsafe fn from_utf8_unchecked(buf: &'a mut [u8], len: usize) -> Self {
        Self::new_unchecked(SliceVec::from_slice_len(buf, len))
    }

    /// Return a mutable reference to the inner `SliceVec`.
    ///
    /// # Safety
    /// The data in the buffer must always remain valid UTF-8.
    pub unsafe fn as_mut_slicevec(&mut self) -> &mut SliceVec<'a, u8> {
        &mut self.0
    }

    /// Return a reference to a UTF-8 `str`.
    pub fn as_str(&self) -> &str {
        // UTF-8 validity has been maintained
        unsafe { str::from_utf8_unchecked(&self.0) }
    }

    /// Return a mutable reference to a UTF-8 `str`.
    pub fn as_mut_str(&mut self) -> &mut str {
        // UTF-8 validity has been maintained
        unsafe { str::from_utf8_unchecked_mut(&mut self.0) }
    }

    /// Return the maximum number of bytes this string can contain.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Set the current string length to zero.
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Set the length to be at most the given number of `u8`.
    ///
    /// # Panics
    /// The new length must be at a character boundary.
    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            assert!(self.is_char_boundary(new_len));
        }
        self.0.truncate(new_len);
    }

    /// Return the last `char` in the string, or `None` if empty.
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.chars().last()?;
        self.0.truncate(self.len() - ch.len_utf8());
        Some(ch)
    }

    /// Append a `char` to the string.
    ///
    /// # Panics
    /// The remaining space must be sufficient.
    pub fn push(&mut self, c: char) {
        match c.len_utf8() {
            1 => self.0.push(c as u8),
            _ => self.push_str(c.encode_utf8(&mut [0; 4])),
        }
    }

    /// Append a `str` to the string.
    ///
    /// # Panics
    /// The remaining space must be sufficient.
    pub fn push_str(&mut self, string: &str) {
        self.0.extend_from_slice(string.as_bytes())
    }

    /// Split the string and return the remainder.
    ///
    /// The current `SliceString` capacity and length are reduced to `at`.
    /// A new `SliceString` is returned containing the data and buffer space starting at `at`.
    ///
    /// # Panics
    /// The split location must be at a character boundary.
    pub fn split_off(&mut self, at: usize) -> SliceString<'a> {
        if at <= self.len() {
            assert!(self.is_char_boundary(at));
        }
        let new = self.0.split_off(at);
        // UTF-8 validity is maintained
        unsafe { Self::new_unchecked(new) }
    }
}

impl<'a> From<SliceString<'a>> for SliceVec<'a, u8> {
    fn from(value: SliceString<'a>) -> Self {
        value.0
    }
}

impl<'a> From<SliceString<'a>> for (&'a mut [u8], usize) {
    fn from(mut value: SliceString<'a>) -> Self {
        let data = value.as_mut_ptr();
        let len = value.capacity();
        // Unfortunately there is no way to destructure SliceVec
        let data = unsafe { core::slice::from_raw_parts_mut(data, len) };
        (data, value.len())
    }
}

impl<'a> TryFrom<&'a mut [u8]> for SliceString<'a> {
    type Error = str::Utf8Error;

    fn try_from(value: &'a mut [u8]) -> Result<Self, Self::Error> {
        Self::from_utf8(value, value.len())
    }
}

impl<'a> TryFrom<SliceVec<'a, u8>> for SliceString<'a> {
    type Error = str::Utf8Error;

    fn try_from(value: SliceVec<'a, u8>) -> Result<Self, Self::Error> {
        str::from_utf8(&value)?;
        // UTF-8 validity has just been checked.
        Ok(unsafe { Self::new_unchecked(value) })
    }
}

impl<'a> ops::Deref for SliceString<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> ops::DerefMut for SliceString<'a> {
    fn deref_mut(&mut self) -> &mut str {
        self.as_mut_str()
    }
}

impl<'a> AsRef<str> for SliceString<'a> {
    fn as_ref(&self) -> &str {
        self
    }
}

impl<'a> AsMut<str> for SliceString<'a> {
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl<'a> AsRef<SliceVec<'a, u8>> for SliceString<'a> {
    fn as_ref(&self) -> &SliceVec<'a, u8> {
        &self.0
    }
}

impl<'a> AsRef<[u8]> for SliceString<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl<'a> fmt::Write for SliceString<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        if self.capacity() < self.len() + s.len() {
            return Err(fmt::Error);
        }
        self.push_str(s);
        Ok(())
    }

    fn write_char(&mut self, c: char) -> Result<(), fmt::Error> {
        if self.capacity() < self.len() + c.len_utf8() {
            return Err(fmt::Error);
        }
        self.push(c);
        Ok(())
    }
}

impl<'a> fmt::Debug for SliceString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Debug>::fmt(self, f)
    }
}

impl<'a> fmt::Display for SliceString<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as fmt::Display>::fmt(self, f)
    }
}

impl<'a> hash::Hash for SliceString<'a> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        <str as hash::Hash>::hash(self, hasher)
    }
}

impl<'a> PartialEq for SliceString<'a> {
    fn eq(&self, rhs: &SliceString<'a>) -> bool {
        str::eq(&**self, &**rhs)
    }
}

// SliceString<'a> == str
impl<'a> PartialEq<str> for SliceString<'a> {
    fn eq(&self, other: &str) -> bool {
        str::eq(&self[..], other)
    }
}

// SliceString<'a> == &'str
impl<'a> PartialEq<&str> for SliceString<'a> {
    fn eq(&self, other: &&str) -> bool {
        str::eq(&self[..], &other[..])
    }
}

// str == SliceString<'a>
impl<'a> PartialEq<SliceString<'a>> for str {
    fn eq(&self, other: &SliceString<'a>) -> bool {
        str::eq(self, &other[..])
    }
}

// &'str == SliceString<'a>
impl<'a> PartialEq<SliceString<'a>> for &str {
    fn eq(&self, other: &SliceString<'a>) -> bool {
        str::eq(&self[..], &other[..])
    }
}

impl<'a> Eq for SliceString<'a> {}

impl<'a> PartialOrd for SliceString<'a> {
    fn partial_cmp(&self, other: &SliceString<'a>) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<'a> Ord for SliceString<'a> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(&**self, &**other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;

    #[test]
    fn new() {
        let mut buf = [0u8; 16];
        let mut s = SliceString::new(&mut buf[..]);
        assert_eq!(0, s.len());
        assert_eq!(s.capacity(), 16);

        s.push_str("Hello world!");
        assert_eq!(s.as_str(), "Hello world!");

        assert!(!s.is_empty());
        s.clear();
        assert_eq!(s.len(), 0);

        s.push_str("foo");
        s.truncate(2);
        assert_eq!(s.len(), 2);
        assert_eq!(s.as_str(), "fo");

        s.push('r');
        assert_eq!(s.as_str(), "for");

        s.write_str("oooooooooooooooooooooo").unwrap_err();

        let mut a = s.split_off(2);
        assert_eq!(s.as_str(), "fo");
        assert_eq!(a.as_str(), "r");

        a.push_str("ab");
        s.push_str("");
        s.write_char('o').unwrap_err();
        assert_eq!(s.capacity(), 2);
        assert_eq!(a.capacity(), 16 - 2);

        let r = unsafe { s.as_mut_slicevec() };
        assert_eq!(r.as_ref(), "fo".as_bytes());

        assert_eq!(s.pop().unwrap(), 'o');
        assert_eq!(s.pop().unwrap(), 'f');
    }

    #[test]
    #[should_panic]
    fn panic_push() {
        let mut buf = [0u8; 1];
        let mut s = SliceString::new(&mut buf[..]);
        s.push('f');
        s.push('o');
    }

    #[test]
    #[should_panic]
    fn panic_push_str() {
        let mut buf = [0u8; 1];
        let mut s = SliceString::new(&mut buf[..]);
        s.push_str("fo");
    }

    #[test]
    fn cmp() {
        let mut b1 = "abcd".as_bytes().to_owned();
        let s1 = SliceString::try_from(&mut b1[..]).unwrap();
        let mut b2 = "zzzz".as_bytes().to_owned();
        let s2 = SliceString::try_from(&mut b2[..]).unwrap();
        assert!(s1 < s2);
    }

    #[test]
    fn disp() {
        let mut b1 = "abcd".as_bytes().to_owned();
        let s1 = SliceString::try_from(&mut b1[..]).unwrap();
        let mut s = String::new();
        write!(s, "{}", s1).unwrap();
        assert_eq!("abcd", s);
    }

    #[test]
    fn pop_uenc() {
        let mut b = "éé".as_bytes().to_owned();
        assert_eq!(b.len(), 2 + 3);
        let mut s = SliceString::try_from(&mut b[..]).unwrap();
        assert_eq!(s.len(), 2 + 3);
        assert_eq!(s.pop().unwrap(), '\u{0301}');
        assert_eq!(s.len(), 2 + 1);
        assert_eq!(s.pop().unwrap(), 'e');
        assert_eq!(s.len(), 2);
        assert_eq!(s.pop().unwrap(), 'é');
        assert_eq!(s.len(), 0);
        s.push('ö');
        s.push_str("ü");
        assert_eq!(s.as_str(), "öü");
    }
}
