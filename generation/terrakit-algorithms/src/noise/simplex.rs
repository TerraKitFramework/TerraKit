//! Coherent deterministic simplex-noise implementations.

use std::f64::consts::FRAC_1_SQRT_2;

use terrakit_core::GenerationSeed;

use crate::{
    AlgorithmError,
    hash::{hash_2d, hash_3d},
};

use super::{Noise2D, Noise3D};

const F2: f64 = 0.366_025_403_784_438_6;
const G2: f64 = 0.211_324_865_405_187_13;
const F3: f64 = 1.0 / 3.0;
const G3: f64 = 1.0 / 6.0;

/// Coherent deterministic 2D simplex noise.
///
/// `SimplexNoise2D` hashes skewed simplex lattice coordinates into
/// deterministic gradient directions. Negative coordinates are supported.
/// Finite inputs return finite signed values roughly in `-1.0..=1.0`;
/// non-finite inputs return `f32::NAN` through the raw [`Noise2D`] trait.
#[derive(Debug, Clone, Copy)]
pub struct SimplexNoise2D {
    seed: GenerationSeed,
}

impl SimplexNoise2D {
    /// Creates simplex noise from a deterministic generation seed.
    pub const fn new(seed: GenerationSeed) -> Self {
        Self { seed }
    }

    /// Returns the seed used by this noise sampler.
    pub const fn seed(&self) -> GenerationSeed {
        self.seed
    }

    /// Validates coordinates and samples the 2D simplex-noise function.
    pub fn try_sample(&self, x: f64, y: f64) -> Result<f32, AlgorithmError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        Ok(self.sample(x, y))
    }
}

impl Noise2D for SimplexNoise2D {
    fn sample(&self, x: f64, y: f64) -> f32 {
        if !x.is_finite() || !y.is_finite() {
            return f32::NAN;
        }

        let skew = (x + y) * F2;
        let i = (x + skew).floor();
        let j = (y + skew).floor();
        let unskew = (i + j) * G2;

        let x0 = x - (i - unskew);
        let y0 = y - (j - unskew);

        let (i1, j1) = if x0 > y0 {
            (1_i64, 0_i64)
        } else {
            (0_i64, 1_i64)
        };

        let x1 = x0 - i1 as f64 + G2;
        let y1 = y0 - j1 as f64 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        let ii = i as i64;
        let jj = j as i64;

        let n0 = simplex_corner_2d(self.seed, ii, jj, x0, y0);
        let n1 = simplex_corner_2d(self.seed, ii.wrapping_add(i1), jj.wrapping_add(j1), x1, y1);
        let n2 = simplex_corner_2d(self.seed, ii.wrapping_add(1), jj.wrapping_add(1), x2, y2);

        (70.0 * (n0 + n1 + n2)) as f32
    }
}

/// Coherent deterministic 3D simplex noise.
///
/// `SimplexNoise3D` hashes skewed simplex lattice coordinates into
/// deterministic gradient directions. Negative coordinates are supported.
/// Finite inputs return finite signed values roughly in `-1.0..=1.0`;
/// non-finite inputs return `f32::NAN` through the raw [`Noise3D`] trait.
#[derive(Debug, Clone, Copy)]
pub struct SimplexNoise3D {
    seed: GenerationSeed,
}

impl SimplexNoise3D {
    /// Creates simplex noise from a deterministic generation seed.
    pub const fn new(seed: GenerationSeed) -> Self {
        Self { seed }
    }

    /// Returns the seed used by this noise sampler.
    pub const fn seed(&self) -> GenerationSeed {
        self.seed
    }

    /// Validates coordinates and samples the 3D simplex-noise function.
    pub fn try_sample(&self, x: f64, y: f64, z: f64) -> Result<f32, AlgorithmError> {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        Ok(self.sample(x, y, z))
    }
}

impl Noise3D for SimplexNoise3D {
    fn sample(&self, x: f64, y: f64, z: f64) -> f32 {
        if !x.is_finite() || !y.is_finite() || !z.is_finite() {
            return f32::NAN;
        }

        let skew = (x + y + z) * F3;
        let i = (x + skew).floor() as i64;
        let j = (y + skew).floor() as i64;
        let k = (z + skew).floor() as i64;
        let unskew = (i as f64 + j as f64 + k as f64) * G3;

        let x0 = x - (i as f64 - unskew);
        let y0 = y - (j as f64 - unskew);
        let z0 = z - (k as f64 - unskew);

        let (i1, j1, k1, i2, j2, k2) = simplex_offsets_3d(x0, y0, z0);

        let x1 = x0 - i1 as f64 + G3;
        let y1 = y0 - j1 as f64 + G3;
        let z1 = z0 - k1 as f64 + G3;

        let x2 = x0 - i2 as f64 + 2.0 * G3;
        let y2 = y0 - j2 as f64 + 2.0 * G3;
        let z2 = z0 - k2 as f64 + 2.0 * G3;

        let x3 = x0 - 1.0 + 3.0 * G3;
        let y3 = y0 - 1.0 + 3.0 * G3;
        let z3 = z0 - 1.0 + 3.0 * G3;

        let n0 = simplex_corner_3d(self.seed, i, j, k, x0, y0, z0);
        let n1 = simplex_corner_3d(
            self.seed,
            i.wrapping_add(i1),
            j.wrapping_add(j1),
            k.wrapping_add(k1),
            x1,
            y1,
            z1,
        );
        let n2 = simplex_corner_3d(
            self.seed,
            i.wrapping_add(i2),
            j.wrapping_add(j2),
            k.wrapping_add(k2),
            x2,
            y2,
            z2,
        );
        let n3 = simplex_corner_3d(
            self.seed,
            i.wrapping_add(1),
            j.wrapping_add(1),
            k.wrapping_add(1),
            x3,
            y3,
            z3,
        );

        (32.0 * (n0 + n1 + n2 + n3)) as f32
    }
}

fn simplex_offsets_3d(x0: f64, y0: f64, z0: f64) -> (i64, i64, i64, i64, i64, i64) {
    if x0 >= y0 {
        if y0 >= z0 {
            (1, 0, 0, 1, 1, 0)
        } else if x0 >= z0 {
            (1, 0, 0, 1, 0, 1)
        } else {
            (0, 0, 1, 1, 0, 1)
        }
    } else if y0 < z0 {
        (0, 0, 1, 0, 1, 1)
    } else if x0 < z0 {
        (0, 1, 0, 0, 1, 1)
    } else {
        (0, 1, 0, 1, 1, 0)
    }
}

fn simplex_corner_2d(seed: GenerationSeed, x: i64, y: i64, dx: f64, dy: f64) -> f64 {
    let falloff = 0.5 - dx * dx - dy * dy;
    if falloff <= 0.0 {
        return 0.0;
    }

    let gradient = gradient_2d(seed, x, y);
    let falloff_squared = falloff * falloff;

    falloff_squared * falloff_squared * (gradient.0 * dx + gradient.1 * dy)
}

fn simplex_corner_3d(
    seed: GenerationSeed,
    x: i64,
    y: i64,
    z: i64,
    dx: f64,
    dy: f64,
    dz: f64,
) -> f64 {
    let falloff = 0.6 - dx * dx - dy * dy - dz * dz;
    if falloff <= 0.0 {
        return 0.0;
    }

    let gradient = gradient_3d(seed, x, y, z);
    let falloff_squared = falloff * falloff;

    falloff_squared * falloff_squared * (gradient.0 * dx + gradient.1 * dy + gradient.2 * dz)
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

    GRADIENTS[(hash_2d(seed, x, y) as usize) & 7]
}

fn gradient_3d(seed: GenerationSeed, x: i64, y: i64, z: i64) -> (f64, f64, f64) {
    const GRADIENTS: [(f64, f64, f64); 12] = [
        (1.0, 1.0, 0.0),
        (-1.0, 1.0, 0.0),
        (1.0, -1.0, 0.0),
        (-1.0, -1.0, 0.0),
        (1.0, 0.0, 1.0),
        (-1.0, 0.0, 1.0),
        (1.0, 0.0, -1.0),
        (-1.0, 0.0, -1.0),
        (0.0, 1.0, 1.0),
        (0.0, -1.0, 1.0),
        (0.0, 1.0, -1.0),
        (0.0, -1.0, -1.0),
    ];

    GRADIENTS[(hash_3d(seed, x, y, z) as usize) % GRADIENTS.len()]
}
