//! Compact and high-precision vector primitives used by TerraKit.
//!
//! `f32` vectors are intended for dense mesh attributes and local data.
//! `f64` vectors are intended for world-space coordinates, spacing, and
//! transforms where precision matters.

mod vector;

pub use vector::{Vector2F32, Vector2F64, Vector3F32, Vector3F64};
