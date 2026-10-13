//! Coherent deterministic Worley/cellular-noise implementations.

use terrakit_core::GenerationSeed;

use crate::{
    AlgorithmError,
    hash::{hash_2d, hash_3d, hash_to_unit_f32},
};

use super::{Noise2D, Noise3D};

/// Coherent deterministic 2D Worley/cellular noise.
///
/// `WorleyNoise2D` places one deterministic feature point in each integer
/// cell, measures Euclidean distance to the nearest feature point in the
/// surrounding cells, clamps that distance to `0.0..=1.0`, then remaps it to
/// `-1.0..=1.0`. Feature points therefore produce values near `-1.0`, while
/// samples far from feature points produce values near `1.0`. Negative
/// coordinates are supported. Non-finite inputs return `f32::NAN` through the
/// raw [`Noise2D`] trait.
#[derive(Debug, Clone, Copy)]
pub struct WorleyNoise2D {
    seed: GenerationSeed,
}

impl WorleyNoise2D {
    /// Creates Worley noise from a deterministic generation seed.
    pub const fn new(seed: GenerationSeed) -> Self {
        Self { seed }
    }

    /// Returns the seed used by this noise sampler.
    pub const fn seed(&self) -> GenerationSeed {
        self.seed
    }

    /// Validates coordinates and samples the 2D Worley-noise function.
    pub fn try_sample(&self, x: f64, y: f64) -> Result<f32, AlgorithmError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        Ok(self.sample(x, y))
    }
}

impl Noise2D for WorleyNoise2D {
    fn sample(&self, x: f64, y: f64) -> f32 {
        if !x.is_finite() || !y.is_finite() {
            return f32::NAN;
        }

        let cell_x = x.floor() as i64;
        let cell_y = y.floor() as i64;
        let mut nearest_squared = f64::INFINITY;

        for offset_y in -1_i64..=1 {
            for offset_x in -1_i64..=1 {
                let neighbor_x = cell_x.wrapping_add(offset_x);
                let neighbor_y = cell_y.wrapping_add(offset_y);
                let point = feature_point_2d(self.seed, neighbor_x, neighbor_y);

                let dx = x - (neighbor_x as f64 + point.0);
                let dy = y - (neighbor_y as f64 + point.1);

                nearest_squared = nearest_squared.min(dx * dx + dy * dy);
            }
        }

        signed_distance(nearest_squared)
    }
}

/// Coherent deterministic 3D Worley/cellular noise.
///
/// `WorleyNoise3D` places one deterministic feature point in each integer
/// cell, measures Euclidean distance to the nearest feature point in the
/// surrounding cells, clamps that distance to `0.0..=1.0`, then remaps it to
/// `-1.0..=1.0`. Feature points therefore produce values near `-1.0`, while
/// samples far from feature points produce values near `1.0`. Negative
/// coordinates are supported. Non-finite inputs return `f32::NAN` through the
/// raw [`Noise3D`] trait.
#[derive(Debug, Clone, Copy)]
pub struct WorleyNoise3D {
    seed: GenerationSeed,
}

impl WorleyNoise3D {
    /// Creates Worley noise from a deterministic generation seed.
    pub const fn new(seed: GenerationSeed) -> Self {
        Self { seed }
    }

    /// Returns the seed used by this noise sampler.
    pub const fn seed(&self) -> GenerationSeed {
        self.seed
    }

    /// Validates coordinates and samples the 3D Worley-noise function.
    pub fn try_sample(&self, x: f64, y: f64, z: f64) -> Result<f32, AlgorithmError> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        Ok(self.sample(x, y, z))
    }
}

impl Noise3D for WorleyNoise3D {
    fn sample(&self, x: f64, y: f64, z: f64) -> f32 {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return f32::NAN;
        }

        let cell_x = x.floor() as i64;
        let cell_y = y.floor() as i64;
        let cell_z = z.floor() as i64;
        let mut nearest_squared = f64::INFINITY;

        for offset_z in -1_i64..=1 {
            for offset_y in -1_i64..=1 {
                for offset_x in -1_i64..=1 {
                    let neighbor_x = cell_x.wrapping_add(offset_x);
                    let neighbor_y = cell_y.wrapping_add(offset_y);
                    let neighbor_z = cell_z.wrapping_add(offset_z);
                    let point = feature_point_3d(self.seed, neighbor_x, neighbor_y, neighbor_z);

                    let dx = x - (neighbor_x as f64 + point.0);
                    let dy = y - (neighbor_y as f64 + point.1);
                    let dz = z - (neighbor_z as f64 + point.2);

                    nearest_squared = nearest_squared.min(dx * dx + dy * dy + dz * dz);
                }
            }
        }

        signed_distance(nearest_squared)
    }
}

fn feature_point_2d(seed: GenerationSeed, x: i64, y: i64) -> (f64, f64) {
    let first = hash_2d(seed, x, y);
    let second = hash_2d(
        seed,
        x.wrapping_add(0x9E37_79B9),
        y.wrapping_add(0x7F4A_7C15),
    );

    (hash_unit(first), hash_unit(second))
}

fn feature_point_3d(seed: GenerationSeed, x: i64, y: i64, z: i64) -> (f64, f64, f64) {
    let first = hash_3d(seed, x, y, z);
    let second = hash_3d(
        seed,
        x.wrapping_add(0x9E37_79B9),
        y.wrapping_add(0x7F4A_7C15),
        z.wrapping_add(0x94D0_49BB),
    );
    let third = hash_3d(
        seed,
        x.wrapping_sub(0x7F4A_7C15),
        y.wrapping_sub(0x94D0_49BB),
        z.wrapping_sub(0x9E37_79B9),
    );

    (hash_unit(first), hash_unit(second), hash_unit(third))
}

fn hash_unit(hash: u64) -> f64 {
    f64::from(hash_to_unit_f32(hash))
}

fn signed_distance(nearest_squared: f64) -> f32 {
    ((nearest_squared.sqrt().min(1.0) * 2.0) - 1.0) as f32
}
