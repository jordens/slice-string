#![no_std]
use core::{fmt, hash, ops, str};
use tinyvec::SliceVec;

#[repr(transparent)]
pub struct SliceString<'a>(SliceVec<'a, u8>);

impl<'a> SliceString<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        unsafe { Self::from_utf8_unchecked(buf, 0) }
    }

    pub unsafe fn new_unchecked(buf: SliceVec<'a, u8>) -> Self {
        Self(buf)
    }

    pub fn from_utf8(buf: &'a mut [u8], len: usize) -> Result<Self, str::Utf8Error> {
        str::from_utf8(&buf[..len])?;
        Ok(unsafe { Self::from_utf8_unchecked(buf, len) })
    }

    pub unsafe fn from_utf8_unchecked(buf: &'a mut [u8], len: usize) -> Self {
        Self::new_unchecked(SliceVec::from_slice_len(buf, len))
    }

    pub unsafe fn as_mut_slicevec<'b>(&'b mut self) -> &'a mut SliceVec<'b, u8> {
        &mut self.0
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }

    pub fn as_mut_str(&mut self) -> &mut str {
        unsafe { str::from_utf8_unchecked_mut(&mut self.0) }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn truncate(&mut self, new_len: usize) {
        if new_len <= self.len() {
            assert!(self.is_char_boundary(new_len));
            self.0.truncate(new_len);
        }
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
            Ok(self.0.push(c as u8))
        } else {
            let mut buf = [0; 4];
            c.encode_utf8(&mut buf);
            Ok(self.0.extend_from_slice(&buf))
        }
    }

    pub fn push_str(&mut self, string: &str) -> Result<(), ()> {
        let bytes = string.as_bytes();
        if self.capacity() < self.len() + bytes.len() {
            return Err(());
        }
        Ok(self.0.extend_from_slice(bytes))
    }

    pub fn split_off<'b>(&'b mut self, at: usize) -> SliceString<'a> {
        if at <= self.len() {
            assert!(self.is_char_boundary(at));
        }
        let new = self.0.split_off(at);
        unsafe { Self::new_unchecked(new) }
    }
}

impl<'a> Default for SliceString<'a> {
    fn default() -> Self {
        Self::new(&mut [])
    }
}

impl<'a> From<SliceString<'a>> for SliceVec<'a, u8> {
    fn from(other: SliceString<'a>) -> Self {
        other.0
    }
}

// impl<'a> From<SliceString<'a>> for &mut [u8] {
//    fn from(other: SliceString<'a>) -> Self {
//        other.0.as_mut()
//    }
// }

impl<'a> TryFrom<&'a mut [u8]> for SliceString<'a> {
    type Error = str::Utf8Error;

    fn try_from(other: &'a mut [u8]) -> Result<Self, Self::Error> {
        Self::from_utf8(other, other.len())
    }
}

impl<'a> TryFrom<SliceVec<'a, u8>> for SliceString<'a> {
    type Error = str::Utf8Error;

    fn try_from(other: SliceVec<'a, u8>) -> Result<Self, Self::Error> {
        str::from_utf8(&other)?;
        Ok(unsafe { Self::new_unchecked(other) })
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

// impl<'a> core::borrow::Borrow<[u8]> for SliceString<'a> {
//     fn borrow(&self) -> &[u8] {
//         self.as_ref()
//     }
// }
//
// impl<'a> core::borrow::Borrow<SliceVec<'a, u8>> for SliceString<'a> {
//     fn borrow(&self) -> &SliceVec<'a, u8> {
//         &self.0
//     }
// }
//
// impl<'a> core::borrow::Borrow<str> for SliceString<'a> {
//     fn borrow(&self) -> &str {
//         self
//     }
// }
//
// impl<'a> core::borrow::BorrowMut<str> for SliceString<'a> {
//     fn borrow_mut(&mut self) -> &mut str {
//         self
//     }
// }

impl<'a> fmt::Write for SliceString<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        self.push_str(s).map_err(|_| fmt::Error)
    }

    fn write_char(&mut self, c: char) -> Result<(), fmt::Error> {
        self.push(c).map_err(|_| fmt::Error)
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

    fn ne(&self, rhs: &SliceString<'a>) -> bool {
        str::ne(&**self, &**rhs)
    }
}

// SliceString<'a> == str
impl<'a> PartialEq<str> for SliceString<'a> {
    fn eq(&self, other: &str) -> bool {
        str::eq(&self[..], &other[..])
    }
    fn ne(&self, other: &str) -> bool {
        str::ne(&self[..], &other[..])
    }
}

// SliceString<'a> == &'str
impl<'a> PartialEq<&str> for SliceString<'a> {
    fn eq(&self, other: &&str) -> bool {
        str::eq(&self[..], &other[..])
    }
    fn ne(&self, other: &&str) -> bool {
        str::ne(&self[..], &other[..])
    }
}

// str == SliceString<'a>
impl<'a> PartialEq<SliceString<'a>> for str {
    fn eq(&self, other: &SliceString<'a>) -> bool {
        str::eq(&self[..], &other[..])
    }
    fn ne(&self, other: &SliceString<'a>) -> bool {
        str::ne(&self[..], &other[..])
    }
}

// &'str == SliceString<'a>
impl<'a> PartialEq<SliceString<'a>> for &str {
    fn eq(&self, other: &SliceString<'a>) -> bool {
        str::eq(&self[..], &other[..])
    }
    fn ne(&self, other: &SliceString<'a>) -> bool {
        str::ne(&self[..], &other[..])
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

    #[test]
    fn new() {
        let mut buf = [0u8; 32];
        let mut s = SliceString::new(&mut buf[..]);
        assert_eq!(0, s.len());
        s.push_str("Hello world!").unwrap();
        assert_eq!(s.as_str(), "Hello world!");

        let _r = unsafe { s.as_mut_slicevec() };
    }
}
