//! The XBF prelude imports the various conversions traits to make creating xbf types as ergonomic
//! as possible. The intention is to include `use xbf_rs::prelude::*;` and have easy access to the
//! majority of things you'll need.
pub use crate::{NativeToXbfPrimitive, XbfMetadataUpcast, XbfTypeUpcast};
