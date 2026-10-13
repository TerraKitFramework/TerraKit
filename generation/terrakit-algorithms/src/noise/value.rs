//! Coherent deterministic value-noise implementations.

use terrakit_core::GenerationSeed;

use crate::{
    AlgorithmError,
    hash::{hash_2d, hash_3d, hash_to_signed_f32},
    interpolation::{lerp_f64, smootherstep},
};

use super::{Noise2D, Noise3D};

/// Coherent deterministic 2D value noise.
///
/// `ValueNoise2D` hashes the surrounding integer lattice coordinates, converts
/// each corner to a signed scalar, and interpolates with the quintic
/// [`smootherstep`] curve. Negative coordinates are supported. Finite inputs
/// return finite values approximately in `-1.0..=1.0`; non-finite inputs return
/// `f32::NAN` through the raw [`Noise2D`] trait.
#[derive(Debug, Clone, Copy)]
pub struct ValueNoise2D {
    seed: GenerationSeed,
}

impl ValueNoise2D {
    /// Creates value noise from a deterministic generation seed.
    pub const fn new(seed: GenerationSeed) -> Self {
        Self { seed }
    }

    /// Returns the seed used by this noise sampler.
    pub const fn seed(&self) -> GenerationSeed {
        self.seed
    }

    /// Validates coordinates and samples the 2D value-noise function.
    pub fn try_sample(&self, x: f64, y: f64) -> Result<f32, AlgorithmError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        Ok(self.sample(x, y))
    }
}

impl Noise2D for ValueNoise2D {
    fn sample(&self, x: f64, y: f64) -> f32 {
        if !x.is_finite() || !y.is_finite() {
            return f32::NAN;
        }

        let x_floor = x.floor();
        let y_floor = y.floor();

        let x0 = x_floor as i64;
        let y0 = y_floor as i64;
        let x1 = x0.wrapping_add(1);
        let y1 = y0.wrapping_add(1);

        let tx = smootherstep(x - x_floor);
        let ty = smootherstep(y - y_floor);

        let v00 = corner_2d(self.seed, x0, y0);
        let v10 = corner_2d(self.seed, x1, y0);
        let v01 = corner_2d(self.seed, x0, y1);
        let v11 = corner_2d(self.seed, x1, y1);

        let bottom = lerp_f64(v00, v10, tx);
        let top = lerp_f64(v01, v11, tx);

        lerp_f64(bottom, top, ty) as f32
    }
}

/// Coherent deterministic 3D value noise.
///
/// `ValueNoise3D` hashes the surrounding integer lattice coordinates, converts
/// each corner to a signed scalar, and interpolates with the quintic
/// [`smootherstep`] curve. Negative coordinates are supported. Finite inputs
/// return finite values approximately in `-1.0..=1.0`; non-finite inputs return
/// `f32::NAN` through the raw [`Noise3D`] trait.
#[derive(Debug, Clone, Copy)]
pub struct ValueNoise3D {
    seed: GenerationSeed,
}

impl ValueNoise3D {
    /// Creates value noise from a deterministic generation seed.
    pub const fn new(seed: GenerationSeed) -> Self {
        Self { seed }
    }

    /// Returns the seed used by this noise sampler.
    pub const fn seed(&self) -> GenerationSeed {
        self.seed
    }

    /// Validates coordinates and samples the 3D value-noise function.
    pub fn try_sample(&self, x: f64, y: f64, z: f64) -> Result<f32, AlgorithmError> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        Ok(self.sample(x, y, z))
    }
}

impl Noise3D for ValueNoise3D {
    fn sample(&self, x: f64, y: f64, z: f64) -> f32 {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return f32::NAN;
        }

        let x_floor = x.floor();
        let y_floor = y.floor();
        let z_floor = z.floor();

        let x0 = x_floor as i64;
        let y0 = y_floor as i64;
        let z0 = z_floor as i64;
        let x1 = x0.wrapping_add(1);
        let y1 = y0.wrapping_add(1);
        let z1 = z0.wrapping_add(1);

        let tx = smootherstep(x - x_floor);
        let ty = smootherstep(y - y_floor);
        let tz = smootherstep(z - z_floor);

        let v000 = corner_3d(self.seed, x0, y0, z0);
        let v100 = corner_3d(self.seed, x1, y0, z0);
        let v010 = corner_3d(self.seed, x0, y1, z0);
        let v110 = corner_3d(self.seed, x1, y1, z0);
        let v001 = corner_3d(self.seed, x0, y0, z1);
        let v101 = corner_3d(self.seed, x1, y0, z1);
        let v011 = corner_3d(self.seed, x0, y1, z1);
        let v111 = corner_3d(self.seed, x1, y1, z1);

        let x00 = lerp_f64(v000, v100, tx);
        let x10 = lerp_f64(v010, v110, tx);
        let x01 = lerp_f64(v001, v101, tx);
        let x11 = lerp_f64(v011, v111, tx);

        let y0_value = lerp_f64(x00, x10, ty);
        let y1_value = lerp_f64(x01, x11, ty);

        lerp_f64(y0_value, y1_value, tz) as f32
    }
}

fn corner_2d(seed: GenerationSeed, x: i64, y: i64) -> f64 {
    hash_to_signed_f32(hash_2d(seed, x, y)) as f64
}

fn corner_3d(seed: GenerationSeed, x: i64, y: i64, z: i64) -> f64 {
    hash_to_signed_f32(hash_3d(seed, x, y, z)) as f64
}
