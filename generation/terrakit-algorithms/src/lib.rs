//! Deterministic procedural-generation algorithms for TerraKit.
//!
//! This crate owns reusable algorithms that operate on numbers and
//! algorithm-space coordinates. It intentionally does not depend on pipeline
//! execution, runtime orchestration, engine integration, resource ownership,
//! dynamic plugins, or C ABI types.
//!
//! The built-in algorithms are deterministic across supported platforms when
//! called with the same [`terrakit_core::GenerationSeed`], coordinates, and
//! settings. Hashing and noise functions use explicit wrapping arithmetic and
//! are intended for procedural generation, not cryptography.
//!
//! # Example
//!
//! ```
//! use terrakit_algorithms::noise::{
//!     fractal_sample_2d, FractalSettings, NoiseAlgorithm,
//! };
//! use terrakit_core::GenerationSeed;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let noise = NoiseAlgorithm::Value.create_2d(GenerationSeed::new(1234));
//!
//! let value = fractal_sample_2d(
//!     &noise,
//!     FractalSettings::default(),
//!     10.0,
//!     20.0,
//! )?;
//!
//! assert!(value.is_finite());
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

/// Algorithm-level validation and sampling errors.
mod error;

/// Deterministic coordinate hashing helpers.
pub mod hash;
/// Interpolation curves and linear interpolation helpers.
pub mod interpolation;
/// Mesh-generation helper functions for regular terrain grids.
pub mod mesh;
/// Noise traits, built-in samplers, and fractal sampling helpers.
pub mod noise;
/// Object scattering options.
pub mod scattering;

pub use error::AlgorithmError;
