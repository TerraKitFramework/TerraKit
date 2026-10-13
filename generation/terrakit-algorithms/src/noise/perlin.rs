//! Coherent deterministic Perlin-noise implementations.

use std::f64::consts::{FRAC_1_SQRT_2, SQRT_2};

use terrakit_core::GenerationSeed;

use crate::{
    AlgorithmError,
    hash::{hash_2d, hash_3d},
    interpolation::{lerp_f64, smootherstep},
};

use super::{Noise2D, Noise3D};

/// Coherent deterministic 2D Perlin noise.
///
/// `PerlinNoise2D` hashes integer lattice coordinates into deterministic
/// gradient directions and interpolates their dot products with the quintic
/// [`smootherstep`] curve. Negative coordinates are supported. Finite inputs
/// return finite signed values roughly in `-1.0..=1.0`; non-finite inputs
/// return `f32::NAN` through the raw [`Noise2D`] trait.
#[derive(Debug, Clone, Copy)]
pub struct PerlinNoise2D {
    seed: GenerationSeed,
}

impl PerlinNoise2D {
    /// Creates Perlin noise from a deterministic generation seed.
    pub const fn new(seed: GenerationSeed) -> Self {
        Self { seed }
    }

    /// Returns the seed used by this noise sampler.
    pub const fn seed(&self) -> GenerationSeed {
        self.seed
    }

    /// Validates coordinates and samples the 2D Perlin-noise function.
    pub fn try_sample(&self, x: f64, y: f64) -> Result<f32, AlgorithmError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        Ok(self.sample(x, y))
    }
}

impl Noise2D for PerlinNoise2D {
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

        let tx = x - x_floor;
        let ty = y - y_floor;

        let g00 = gradient_2d(self.seed, x0, y0);
        let g10 = gradient_2d(self.seed, x1, y0);
        let g01 = gradient_2d(self.seed, x0, y1);
        let g11 = gradient_2d(self.seed, x1, y1);

        let n00 = g00.0 * tx + g00.1 * ty;
        let n10 = g10.0 * (tx - 1.0) + g10.1 * ty;
        let n01 = g01.0 * tx + g01.1 * (ty - 1.0);
        let n11 = g11.0 * (tx - 1.0) + g11.1 * (ty - 1.0);

        let u = smootherstep(tx);
        let v = smootherstep(ty);

        let nx0 = lerp_f64(n00, n10, u);
        let nx1 = lerp_f64(n01, n11, u);

        (lerp_f64(nx0, nx1, v) * SQRT_2) as f32
    }
}

/// Coherent deterministic 3D Perlin noise.
///
/// `PerlinNoise3D` hashes integer lattice coordinates into deterministic
/// gradient directions and interpolates their dot products with the quintic
/// [`smootherstep`] curve. Negative coordinates are supported. Finite inputs
/// return finite signed values roughly in `-1.0..=1.0`; non-finite inputs
/// return `f32::NAN` through the raw [`Noise3D`] trait.
#[derive(Debug, Clone, Copy)]
pub struct PerlinNoise3D {
    seed: GenerationSeed,
}

impl PerlinNoise3D {
    /// Creates Perlin noise from a deterministic generation seed.
    pub const fn new(seed: GenerationSeed) -> Self {
        Self { seed }
    }

    /// Returns the seed used by this noise sampler.
    pub const fn seed(&self) -> GenerationSeed {
        self.seed
    }

    /// Validates coordinates and samples the 3D Perlin-noise function.
    pub fn try_sample(&self, x: f64, y: f64, z: f64) -> Result<f32, AlgorithmError> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        Ok(self.sample(x, y, z))
    }
}

impl Noise3D for PerlinNoise3D {
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

        let tx = x - x_floor;
        let ty = y - y_floor;
        let tz = z - z_floor;

        let n000 = dot_gradient_3d(self.seed, x0, y0, z0, tx, ty, tz);
        let n100 = dot_gradient_3d(self.seed, x1, y0, z0, tx - 1.0, ty, tz);
        let n010 = dot_gradient_3d(self.seed, x0, y1, z0, tx, ty - 1.0, tz);
        let n110 = dot_gradient_3d(self.seed, x1, y1, z0, tx - 1.0, ty - 1.0, tz);
        let n001 = dot_gradient_3d(self.seed, x0, y0, z1, tx, ty, tz - 1.0);
        let n101 = dot_gradient_3d(self.seed, x1, y0, z1, tx - 1.0, ty, tz - 1.0);
        let n011 = dot_gradient_3d(self.seed, x0, y1, z1, tx, ty - 1.0, tz - 1.0);
        let n111 = dot_gradient_3d(self.seed, x1, y1, z1, tx - 1.0, ty - 1.0, tz - 1.0);

        let u = smootherstep(tx);
        let v = smootherstep(ty);
        let w = smootherstep(tz);

        let x00 = lerp_f64(n000, n100, u);
        let x10 = lerp_f64(n010, n110, u);
        let x01 = lerp_f64(n001, n101, u);
        let x11 = lerp_f64(n011, n111, u);

        let y0_value = lerp_f64(x00, x10, v);
        let y1_value = lerp_f64(x01, x11, v);

        (lerp_f64(y0_value, y1_value, w) * SQRT_2) as f32
    }
}

fn gradient_2d(seed: GenerationSeed, x: i64, y: i64) -> (f64, f64) {
    const GRADIENTS: [(f64, f64); 8] = [
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, 1.0),
        (0.0, -1.0),
        (FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        (-FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        (FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    ];

    let hash = hash_2d(seed, x, y);
    GRADIENTS[(hash as usize) & 7]
}

fn dot_gradient_3d(seed: GenerationSeed, x: i64, y: i64, z: i64, dx: f64, dy: f64, dz: f64) -> f64 {
    const GRADIENTS: [(f64, f64, f64); 12] = [
        (FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0),
        (-FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0),
        (FRAC_1_SQRT_2, -FRAC_1_SQRT_2, 0.0),
        (-FRAC_1_SQRT_2, -FRAC_1_SQRT_2, 0.0),
        (FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2),
        (-FRAC_1_SQRT_2, 0.0, FRAC_1_SQRT_2),
        (FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2),
        (-FRAC_1_SQRT_2, 0.0, -FRAC_1_SQRT_2),
        (0.0, FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        (0.0, -FRAC_1_SQRT_2, FRAC_1_SQRT_2),
        (0.0, FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
        (0.0, -FRAC_1_SQRT_2, -FRAC_1_SQRT_2),
    ];

    let hash = hash_3d(seed, x, y, z);
    let gradient = GRADIENTS[(hash as usize) % GRADIENTS.len()];

    gradient.0 * dx + gradient.1 * dy + gradient.2 * dz
}
