use super::*;

// primitives in this mod should provide the same no-panic guarantees as the crate `untrusted`

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadError {
    TooShort { rest: usize, expected: usize },
    TooLong { rest: usize },
    // TODO temp solution
    TooLongReadLen(usize),
}

type Result<T> = core::result::Result<T, ReadError>;

pub struct Reader<I> {
    input: I,
    pos: usize,
}

impl<B: AsRef<[u8]>, I: Input<Storage = B>> Reader<I> {
    pub fn new(bytes: B) -> Self {
        Reader { input: bytes.into(), pos: 0 }
    }

    #[inline(always)]
    pub /* const */ fn rest_len(&self) -> usize {
        self.input.len() - self.pos
    }

    pub fn split_out(&mut self, size: usize) -> Result<I> {
        let new_pos = self.pos.checked_add(size)
            .ok_or(ReadError::TooLongReadLen(size))?;
        let ret = self.input.bytes(self.pos..new_pos)
            .ok_or(ReadError::TooShort { rest: self.rest_len(), expected: size })?;
        self.pos = new_pos;
        Ok(ret)
    }

    // copies
    pub fn read_byte(&mut self) -> Result<u8> {
        match self.input.byte(self.pos) {
            Some(b) => {
                // safe from overflow; see https://docs.rs/untrusted/0.9.0/src/untrusted/input.rs.html#39-43
                self.pos += 1;
                Ok(*b)
            },
            None => Err(ReadError::TooShort { rest: 0, expected: 1 })
        }
    }

    pub fn finish(self) -> Result<()> {
        let rest = self.rest_len();
        if rest == 0 {
            Ok(())
        } else {
            Err(ReadError::TooLong { rest })
        }
    }

    pub fn into_rest(mut self) -> I {
        self.split_out(self.rest_len()).unwrap()
    }
}

// primitive derivatives
impl<B: AsRef<[u8]>, I: Input<Storage = B>> Reader<I> {
    // copies
    #[inline]
    pub fn split_out_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self.split_out(N)?.leak_as_array())
    }

    // copies
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let src = self.split_out(buf.len())?;
        if buf.len() == 1 {
            // checked below
            buf[0] = *src.byte(0).unwrap();
        } else {
            buf.copy_from_slice(src.leak().as_ref());
        }
        Ok(())
    }

    #[inline]
    pub fn bytes(&mut self, sz: usize) -> Result<B> {
        Ok(self.split_out(sz)?.leak())
    }

    // copies
    #[inline]
    pub fn bytes_sized<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.split_out_array()
    }
}
