//! This crate is a Rust implementaton of the XBF serialization and deserialization format.
//!
//! The format itself is described [here](https://github.com/XtensibleBinaryFormat/XBF/blob/main/docs/specification.md)

mod base_metadata;
mod base_type;
mod util;
mod xbf_primitive;
mod xbf_struct;
mod xbf_vec;

pub mod prelude;

pub use base_metadata::*;
pub use base_type::*;
pub use xbf_primitive::*;
pub use xbf_struct::*;
pub use xbf_vec::*;
