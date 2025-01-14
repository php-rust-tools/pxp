use std::{mem, ops::Deref};

use crate::ByteString;

#[derive(PartialOrd, PartialEq, Eq, Hash)]
pub struct ByteStr([u8]);

impl ByteStr {
    pub fn new(bytes: &[u8]) -> &Self {
        unsafe { mem::transmute(bytes) }
    }

    pub fn to_bytestring(&self) -> ByteString {
        ByteString::from(self)
    }

    pub fn contains(&self, needle: &[u8]) -> bool {
        self.0.windows(needle.len()).any(|window| window == needle)
    }

    pub fn strip_prefix(&self, prefix: u8) -> &ByteStr {
        let mut start = 0;
        let mut end = self.0.len();

        while start < end && self.0[start] == prefix {
            start += 1;
        }

        while end > start && self.0[end - 1] == prefix {
            end -= 1;
        }

        ByteStr::new(&self.0[start..end])
    }

    pub fn after_first(&self, needle: u8) -> &ByteStr {
        let start = self
            .0
            .iter()
            .position(|&b| b == needle)
            .map_or(self.0.len(), |i| i + 1);

        ByteStr::new(&self.0[start..])
    }

    pub fn before_first(&self, needle: u8) -> &ByteStr {
        let end = self
            .0
            .iter()
            .position(|&b| b == needle)
            .unwrap_or(self.0.len());

        ByteStr::new(&self.0[..end])
    }

    pub fn after_last(&self, needle: u8) -> &ByteStr {
        let start = self
            .0
            .iter()
            .rposition(|&b| b == needle)
            .map_or(0, |i| i + 1);

        ByteStr::new(&self.0[start..])
    }

    pub fn strip_string_quotes(&self) -> &ByteStr {
        if self.0[0] == b'"' && self.0[self.0.len() - 1] == b'"' {
            ByteStr::new(&self.0[1..self.0.len() - 1])
        } else if self.0[0] == b'\'' && self.0[self.0.len() - 1] == b'\'' {
            ByteStr::new(&self.0[1..self.0.len() - 1])
        } else {
            self
        }
    }

    pub fn coagulate(&self, others: &[&ByteStr], with: u8) -> ByteString {
        let mut bytes = self.0.to_vec();

        for other in others {
            bytes.push(with);
            bytes.extend_from_slice(&other.0);
        }

        ByteString::new(bytes)
    }
}

impl std::fmt::Display for ByteStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for &b in &self.0 {
            match b {
                0 => write!(f, "\\0")?,
                b'\n' | b'\r' | b'\t' => write!(f, "{}", b as char)?,
                0x01..=0x19 | 0x7f..=0xff => write!(f, "\\x{:02x}", b)?,
                _ => write!(f, "{}", b as char)?,
            }
        }

        Ok(())
    }
}

impl<'a> From<&'a str> for &'a ByteStr {
    fn from(value: &'a str) -> Self {
        ByteStr::new(value.as_bytes())
    }
}

impl std::fmt::Debug for ByteStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        for &b in &self.0 {
            match b {
                0 => write!(f, "\\0")?,
                b'\n' | b'\r' | b'\t' => write!(f, "{}", b.escape_ascii())?,
                0x01..=0x19 | 0x7f..=0xff => write!(f, "\\x{:02x}", b)?,
                _ => write!(f, "{}", b as char)?,
            }
        }
        write!(f, "\"")?;
        Ok(())
    }
}

impl<const N: usize> PartialEq<[u8; N]> for ByteStr {
    fn eq(&self, other: &[u8; N]) -> bool {
        &self.0 == other
    }
}

impl<const N: usize> PartialEq<[u8; N]> for &ByteStr {
    fn eq(&self, other: &[u8; N]) -> bool {
        &self.0 == other
    }
}

impl<'a> From<&'a [u8]> for &'a ByteStr {
    fn from(value: &'a [u8]) -> Self {
        ByteStr::new(value)
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for &'a ByteStr {
    fn from(bytes: &'a [u8; N]) -> Self {
        ByteStr::new(bytes)
    }
}

impl Deref for ByteStr {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
