//! IR transforms. Each pass takes `&mut Func` and reports how much it changed.
pub mod fold;
pub mod strength;
