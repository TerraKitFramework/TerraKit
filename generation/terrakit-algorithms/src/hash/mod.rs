//! Deterministic coordinate hashing for procedural generation.
//!
//! These helpers support negative coordinates, use explicit wrapping integer
//! arithmetic, allocate no heap memory, and do not use platform-dependent or
//! randomized hashers. They are not cryptographic hashes.

mod coordinate;

pub use coordinate::*;
