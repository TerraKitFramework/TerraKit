//! Object-safe traits for 2D and 3D noise samplers.

/// Object-safe trait for immutable 2D noise sampling.
///
/// Inputs are algorithm-space coordinates. Implementations should support
/// negative coordinates where meaningful and normally return finite values
/// approximately in `-1.0..=1.0`. The trait itself performs no validation so it
/// can stay fast and usable through `dyn Noise2D`.
pub trait Noise2D: Send + Sync {
    /// Samples the noise function at `(x, y)`.
    fn sample(&self, x: f64, y: f64) -> f32;
}

/// Object-safe trait for immutable 3D noise sampling.
///
/// Inputs are algorithm-space coordinates. Implementations should support
/// negative coordinates where meaningful and normally return finite values
/// approximately in `-1.0..=1.0`. The trait itself performs no validation so it
/// can stay fast and usable through `dyn Noise3D`.
pub trait Noise3D: Send + Sync {
    /// Samples the noise function at `(x, y, z)`.
    fn sample(&self, x: f64, y: f64, z: f64) -> f32;
}
