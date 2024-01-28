#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::{boxed::Box, string::String, vec::Vec};

#[cfg(feature = "bytes")]
pub use bytes::{self, Bytes};

mod marker;
pub use marker::*;

mod bytestr;
pub use bytestr::*;

mod input;
pub use input::*;

mod output;
pub use output::*;
