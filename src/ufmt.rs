use crate::SliceString;
use ufmt_write::uWrite;

impl uWrite for SliceString<'_> {
    type Error = ();

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        if self.capacity() < self.len() + s.len() {
            return Err(());
        }

        self.push_str(s);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ufmt::{derive::uDebug, uwrite};

    use crate::SliceString;

    #[derive(uDebug)]
    struct Pair {
        x: u32,
        y: u32,
    }

    #[test]
    fn test_string() {
        let a = 123;
        let b = Pair { x: 0, y: 1234 };

        let mut buf = [0u8; 32];
        let mut s = SliceString::new(&mut buf[..]);
        uwrite!(s, "{} -> {:?}", a, b).unwrap();

        assert_eq!(s, "123 -> Pair { x: 0, y: 1234 }");
    }

    #[test]
    fn test_string_err() {
        let p = Pair { x: 0, y: 1234 };
        let mut buf = [0u8; 4];
        let mut s = SliceString::new(&mut buf[..]);
        assert!(uwrite!(s, "{:?}", p).is_err());
    }
}
