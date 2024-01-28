use super::*;

/// # Safety
/// types impl this should provide the same no-panic guarantees as the crate `untrusted`
pub unsafe trait Input: Sized + From<Self::Storage> + ByteStorage {
    type Storage: AsRef<[u8]>;
    fn byte(&self, pos: usize) -> Option<&u8>;
    fn bytes(&self, range: core::ops::Range<usize>) -> Option<Self>;
    /* const */ fn len(&self) -> usize;
    /* const */ fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn leak(self) -> Self::Storage;
    fn leak_as_array<const N: usize>(self) -> [u8; N];
}

pub struct SliceInput<'a> {
    bytes: &'a [u8],
}

impl<'a> From<&'a [u8]> for SliceInput<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        SliceInput { bytes }
    }
}

unsafe impl<'a> Input for SliceInput<'a> {
    type Storage = &'a [u8];

    #[inline(always)]
    fn byte(&self, pos: usize) -> Option<&u8> {
        self.bytes.get(pos)
    }

    #[inline(always)]
    fn bytes(&self, range: core::ops::Range<usize>) -> Option<Self> {
        self.bytes.get(range).map(|bytes| SliceInput { bytes })
    }

    #[inline(always)]
    /* const */ fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline(always)]
    /* const */ fn leak(self) -> &'a [u8] {
        self.bytes
    }

    #[inline]
    fn leak_as_array<const N: usize>(self) -> [u8; N] {
        let r: &[u8; N] = self.bytes.try_into().unwrap(/* ? */);
        *r
    }
}

#[cfg(feature = "bytes")]
pub struct BytesInput {
    bytes: Bytes,
}

#[cfg(feature = "bytes")]
impl From<Bytes> for BytesInput {
    fn from(bytes: Bytes) -> Self {
        BytesInput { bytes }
    }
}

#[cfg(feature = "bytes")]
unsafe impl Input for BytesInput {
    type Storage = Bytes;

    #[inline(always)]
    fn byte(&self, pos: usize) -> Option<&u8> {
        self.bytes.get(pos)
    }

    #[inline(always)]
    fn bytes(&self, range: core::ops::Range<usize>) -> Option<Self> {
        // from <core::ops::Range<usize> as core::slice::SliceIndex<[T]>>::get
        let core::ops::Range { start, end } = range;
        if start > end || end > self.len() {
            None
        } else {
            Some(BytesInput { bytes: self.bytes.slice(range) })
        }
    }

    #[inline(always)]
    /* const */ fn len(&self) -> usize {
        self.bytes.len()
    }

    #[inline(always)]
    fn leak(self) -> Bytes {
        self.bytes
    }

    // copies
    #[inline]
    fn leak_as_array<const N: usize>(self) -> [u8; N] {
        self.bytes.as_ref().try_into().unwrap(/* ? */)
    }
}
