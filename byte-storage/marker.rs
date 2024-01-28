use super::*;

/// # Safety
/// types impl this should provide the same guarantees as string::StableAsRef
pub unsafe trait ByteStorage {}
unsafe impl<B: ?Sized + ByteStorage> ByteStorage for &'_ B {}
unsafe impl ByteStorage for [u8] {}
unsafe impl ByteStorage for str {}
unsafe impl<const N: usize> ByteStorage for [u8; N] {}
#[cfg(feature = "alloc")]
unsafe impl ByteStorage for alloc::vec::Vec<u8> {}
#[cfg(feature = "alloc")]
unsafe impl ByteStorage for alloc::string::String {}
#[cfg(feature = "bytes")]
unsafe impl ByteStorage for Bytes {}
unsafe impl ByteStorage for input::SliceInput<'_> {}
#[cfg(feature = "bytes")]
unsafe impl ByteStorage for input::BytesInput {}
// types impl Output no need to impl this
