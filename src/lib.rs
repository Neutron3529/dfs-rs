#![feature(associated_type_bounds)]

pub mod indexable;
pub mod proto;

#[cfg(feature = "unsafe")]
pub use crate::indexable::*;
#[cfg(not(feature = "unsafe"))]
pub use crate::proto::*;
