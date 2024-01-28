use crate::*;

/// # Safety
/// types impl this should provide the same guarantees as string::StableAsRef
pub unsafe trait ByteStorage {}
unsafe impl<B: ?Sized + ByteStorage> ByteStorage for &'_ B {}
unsafe impl<B: ?Sized + ByteStorage> ByteStorage for &'_ mut B {}
#[cfg(feature = "alloc")]
unsafe impl<B: ?Sized + ByteStorage> ByteStorage for Box<B> {}
// feature std & impl for Rc & Arc ?
unsafe impl ByteStorage for [u8] {}
unsafe impl ByteStorage for str {}
unsafe impl<const N: usize> ByteStorage for [u8; N] {}
#[cfg(feature = "alloc")]
unsafe impl ByteStorage for Vec<u8> {}
#[cfg(feature = "alloc")]
unsafe impl ByteStorage for String {}
#[cfg(feature = "bytes")]
unsafe impl ByteStorage for Bytes {}
unsafe impl ByteStorage for SliceInput<'_> {}
#[cfg(feature = "bytes")]
unsafe impl ByteStorage for BytesInput {}
// types impl Output no need to impl this
