// TODO impl a more general one outside

use alloc::{string::String, vec::Vec};
use crate::ByteStorage;
#[cfg(feature = "bytes")]
use crate::Bytes;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// currently no need for Default & new()
// TODO impl<B: AsRef<[u8]>, S: AsRef<str>> PartialEq&PartialOrd<S> for ByteStr<B>
// cannot due to similar problem with Value
pub struct ByteStr<B> {
    // Invariant: bytes contains valid UTF-8
    bytes: B,
}

impl<'a> From<&'a str> for ByteStr<&'a [u8]> {
    fn from(value: &'a str) -> Self {
        // Invariant: value is a str so contains valid UTF-8.
        ByteStr { bytes: value.as_bytes() }
    }
}

impl From<String> for ByteStr<Vec<u8>> {
    fn from(value: String) -> Self {
        // Invariant: value is a String so contains valid UTF-8.
        ByteStr { bytes: value.into_bytes() }
    }
}

#[cfg(feature = "bytes")]
impl From<&'static str> for ByteStr<Bytes> {
    /* const */ fn from(value: &'static str) -> Self {
        ByteStr {
            // Invariant: value is a str so contains valid UTF-8.
            bytes: Bytes::from_static(value.as_bytes()),
        }
    }
}

// conflict with above
/*
#[cfg(feature = "bytes")]
impl<'a> From<&'a str> for ByteStr<Bytes> {
    /* const */ fn from(value: &'static str) -> Self {
        ByteStr {
            // Invariant: value is a str so contains valid UTF-8.
            bytes: Bytes::copy_from_slice(value.as_bytes()),
        }
    }
}
*/

impl<B: AsRef<[u8]> + ByteStorage> ByteStr<B> {
    // DO NOT impl TryFrom<B> keeping consistency to std
    // https://internals.rust-lang.org/t/20078/
    pub fn from_utf8(bytes: B) -> Result<Self, core::str::Utf8Error> {
        let _ = core::str::from_utf8(bytes.as_ref())?;
        Ok(ByteStr { bytes })
    }

    /// ## Panics
    /// In a debug build this will panic if `bytes` is not valid UTF-8.
    ///
    /// ## Safety
    /// `bytes` must contain valid UTF-8. In a release build it is undefined
    /// behaviour to call this with `bytes` that is not valid UTF-8.
    pub unsafe fn from_utf8_unchecked(bytes: B) -> ByteStr<B> {
        if cfg!(debug_assertions) {
            match core::str::from_utf8(bytes.as_ref()) {
                Ok(_) => (),
                Err(err) => panic!(
                    "ByteStr::from_utf8_unchecked() with invalid bytes; error = {}, bytes = {:?}",
                    err, bytes.as_ref()
                ),
            }
        }
        // Invariant: assumed by the safety requirements of this function.
        ByteStr { bytes }
    }

    // cannot impl<B> From<ByteStr<B>> for B
    pub fn leak_bytes(self) -> B {
        self.bytes
    }
}

impl<B: AsRef<[u8]>> AsRef<[u8]> for ByteStr<B> {
    fn as_ref(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}

impl<B: AsRef<[u8]> + ByteStorage> AsRef<str> for ByteStr<B> {
    fn as_ref(&self) -> &str {
        let b: &[u8] = self.bytes.as_ref();
        // Safety: the invariant of `bytes` is that it contains valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(b) }
    }
}

impl<B> ByteStr<B> {
    pub fn map_bytes<B2>(self, f: fn(B) -> B2) -> ByteStr<B2> {
        ByteStr { bytes: f(self.bytes) }
    }
}
