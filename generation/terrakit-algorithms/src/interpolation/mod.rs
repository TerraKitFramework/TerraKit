//! Reusable interpolation curves for deterministic algorithms.
//!
//! These helpers do not allocate and do not clamp interpolation parameters
//! unless the function name explicitly says so.

mod curves;

pub use curves::*;
