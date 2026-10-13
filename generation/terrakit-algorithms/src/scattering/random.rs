//! Deterministic local-coordinate scattering helpers.

use terrakit_core::ScatterPoint;

use super::types::{ScatterArea, ScatterError, ScatterSettings};

/// A small deterministic pseudo-random number generator.
#[derive(Debug, Clone, Copy)]
pub struct ScatterRng {
    state: u64,
}

impl ScatterRng {
    /// Creates a new deterministic RNG.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Generates the next random u64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);

        let mut z = self.state;

        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);

        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);

        z ^ (z >> 31)
    }

    /// Generates a random floating-point number in [0, 1).
    pub fn next_f32(&mut self) -> f32 {
        let value = self.next_u64();

        let bits = (value >> 40) as u32;

        bits as f32 / (1u32 << 24) as f32
    }

    /// Generates a floating-point value in [min, max].
    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Generates an integer in [min, max).
    pub fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        if max <= min {
            return min;
        }

        min + (self.next_u64() % (max - min) as u64) as u32
    }
}

/// Generates random scatter points throughout a rectangular area.
pub fn scatter_random(
    settings: ScatterSettings,
    area: ScatterArea,
    count: u32,
) -> Result<Vec<ScatterPoint>, ScatterError> {
    settings.validate()?;
    area.validate()?;

    let mut rng = ScatterRng::new(settings.seed);

    let mut points = Vec::with_capacity(count as usize);

    for _ in 0..count {
        let x = rng.range_f32(area.min_x, area.max_x);
        let z = rng.range_f32(area.min_z, area.max_z);

        let scale = rng.range_f32(settings.min_scale, settings.max_scale);

        let rotation = if settings.random_rotation {
            random_y_rotation(&mut rng)
        } else {
            [0.0, 0.0, 0.0, 1.0]
        };

        points.push(ScatterPoint {
            position: [x, 0.0, z],
            rotation,
            scale: [scale, scale, scale],
            prototype_id: settings.prototype_id,
        });
    }

    Ok(points)
}

/// Generates a random Y-axis quaternion.
pub fn random_y_rotation(rng: &mut ScatterRng) -> [f32; 4] {
    let angle = rng.next_f32() * std::f32::consts::TAU;

    let half = angle * 0.5;

    [0.0, half.sin(), 0.0, half.cos()]
}
