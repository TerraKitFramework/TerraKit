//! Generates engine-independent scattering positions and placement attributes.
//!
//! Algorithms in this module return placement data; they do not instantiate objects.

mod masks;
mod poisson;
mod random;
mod types;

pub use masks::*;
pub use poisson::*;
pub use random::*;
pub use types::*;
