use crate::*;

// TODO should we make no-panic guarantees like reader::Input ?
// before this, Output does not need to be unsafe

pub trait Output: Default {
    type Storage: ByteStorage; // ?
    fn byte(&mut self, n: u8);
    fn bytes<B: AsRef<[u8]>>(&mut self, bytes: B);
    fn leak(self) -> Self::Storage;
}

#[cfg(feature = "alloc")]
#[derive(Default)]
pub struct VecOutput {
    bytes: Vec<u8>,
}

#[cfg(feature = "alloc")]
impl Output for VecOutput {
    type Storage = Vec<u8>;

    #[inline]
    fn byte(&mut self, n: u8) {
        self.bytes.push(n);
    }

    #[inline]
    fn bytes<B: AsRef<[u8]>>(&mut self, bytes: B) {
        self.bytes.extend_from_slice(bytes.as_ref());
    }

    fn leak(self) -> Self::Storage {
        self.bytes
    }
}
