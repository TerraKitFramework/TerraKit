//! Fractal composition over 2D and 3D noise sources.

use crate::AlgorithmError;

use super::{Noise2D, Noise3D};

/// Maximum octave count accepted by [`FractalSettings::validate`].
pub const MAX_FRACTAL_OCTAVES: u8 = 32;

/// Settings for deterministic fractal noise composition.
///
/// Fractal sampling evaluates a source noise function at increasing
/// frequencies and changing amplitudes. Negative amplitude is permitted and
/// flips the contribution sign. Zero amplitude is valid and produces zero when
/// all octave amplitudes are zero. Invalid numeric settings are rejected by
/// [`FractalSettings::validate`] rather than replaced with defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractalSettings {
    /// Number of octaves to evaluate. Must be in `1..=MAX_FRACTAL_OCTAVES`.
    pub octaves: u8,

    /// Initial coordinate multiplier. Must be finite and greater than zero.
    pub frequency: f64,

    /// Per-octave frequency multiplier. Must be finite and greater than zero.
    pub lacunarity: f64,

    /// Per-octave amplitude multiplier. Must be finite and non-negative.
    pub persistence: f32,

    /// Initial amplitude. Must be finite. Negative values are permitted.
    pub amplitude: f32,

    /// When true, divides by accumulated absolute amplitude weight.
    pub normalize: bool,
}

impl Default for FractalSettings {
    fn default() -> Self {
        Self {
            octaves: 4,
            frequency: 0.01,
            lacunarity: 2.0,
            persistence: 0.5,
            amplitude: 1.0,
            normalize: true,
        }
    }
}

impl FractalSettings {
    /// Validates numeric settings and returns the unchanged settings on success.
    ///
    /// Validation rejects zero or excessive octaves, non-finite settings,
    /// non-positive frequency or lacunarity, negative persistence, and settings
    /// that would make octave frequency, octave amplitude, or non-normalized
    /// output scale obviously non-finite.
    pub fn validate(self) -> Result<Self, AlgorithmError> {
        if self.octaves == 0 || self.octaves > MAX_FRACTAL_OCTAVES {
            return Err(AlgorithmError::InvalidOctaveCount);
        }

        if !self.frequency.is_finite() {
            return Err(AlgorithmError::NonFiniteSetting { field: "frequency" });
        }
        if self.frequency <= 0.0 {
            return Err(AlgorithmError::InvalidFrequency);
        }

        if !self.lacunarity.is_finite() {
            return Err(AlgorithmError::NonFiniteSetting {
                field: "lacunarity",
            });
        }
        if self.lacunarity <= 0.0 {
            return Err(AlgorithmError::InvalidLacunarity);
        }

        if !self.persistence.is_finite() {
            return Err(AlgorithmError::NonFiniteSetting {
                field: "persistence",
            });
        }
        if self.persistence < 0.0 {
            return Err(AlgorithmError::InvalidPersistence);
        }

        if !self.amplitude.is_finite() {
            return Err(AlgorithmError::NonFiniteSetting { field: "amplitude" });
        }

        let mut frequency = self.frequency;
        let mut amplitude = self.amplitude;
        let mut weight = 0.0_f64;

        for octave in 0..self.octaves {
            if !frequency.is_finite() {
                return Err(AlgorithmError::InvalidFrequency);
            }
            if !amplitude.is_finite() {
                return Err(AlgorithmError::InvalidAmplitude);
            }

            weight += f64::from(amplitude.abs());
            if !weight.is_finite() {
                return Err(AlgorithmError::InvalidAmplitude);
            }

            if octave + 1 < self.octaves {
                frequency *= self.lacunarity;
                amplitude *= self.persistence;
            }
        }

        if !self.normalize && weight > f64::from(f32::MAX) {
            return Err(AlgorithmError::InvalidAmplitude);
        }

        Ok(self)
    }
}

/// Samples 2D fractal noise with validated settings and finite coordinates.
///
/// Coordinates are algorithm-space values and may be negative. The source noise
/// object is not mutated. When `settings.normalize` is true, the result is
/// divided by accumulated absolute amplitude weight and is normally in roughly
/// `-1.0..=1.0` if the source noise follows that range. When all octave
/// amplitudes are zero, normalized sampling returns `0.0`.
pub fn fractal_sample_2d<N>(
    noise: &N,
    settings: FractalSettings,
    x: f64,
    y: f64,
) -> Result<f32, AlgorithmError>
where
    N: Noise2D + ?Sized,
{
    if !x.is_finite() || !y.is_finite() {
        return Err(AlgorithmError::NonFiniteCoordinate);
    }

    let settings = settings.validate()?;
    let mut state = FractalState::new(settings);

    for _ in 0..settings.octaves {
        let sx = x * state.frequency;
        let sy = y * state.frequency;
        if !sx.is_finite() || !sy.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        state.add_sample(noise.sample(sx, sy))?;
        state.advance(settings);
    }

    state.finish(settings)
}

/// Samples 3D fractal noise with validated settings and finite coordinates.
///
/// Coordinates are algorithm-space values and may be negative. The source noise
/// object is not mutated. When `settings.normalize` is true, the result is
/// divided by accumulated absolute amplitude weight and is normally in roughly
/// `-1.0..=1.0` if the source noise follows that range. When all octave
/// amplitudes are zero, normalized sampling returns `0.0`.
pub fn fractal_sample_3d<N>(
    noise: &N,
    settings: FractalSettings,
    x: f64,
    y: f64,
    z: f64,
) -> Result<f32, AlgorithmError>
where
    N: Noise3D + ?Sized,
{
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return Err(AlgorithmError::NonFiniteCoordinate);
    }

    let settings = settings.validate()?;
    let mut state = FractalState::new(settings);

    for _ in 0..settings.octaves {
        let sx = x * state.frequency;
        let sy = y * state.frequency;
        let sz = z * state.frequency;
        if !sx.is_finite() || !sy.is_finite() || !sz.is_finite() {
            return Err(AlgorithmError::NonFiniteCoordinate);
        }

        state.add_sample(noise.sample(sx, sy, sz))?;
        state.advance(settings);
    }

    state.finish(settings)
}

struct FractalState {
    total: f64,
    frequency: f64,
    amplitude: f32,
    weight: f64,
}

impl FractalState {
    fn new(settings: FractalSettings) -> Self {
        Self {
            total: 0.0,
            frequency: settings.frequency,
            amplitude: settings.amplitude,
            weight: 0.0,
        }
    }

    fn add_sample(&mut self, sample: f32) -> Result<(), AlgorithmError> {
        if !sample.is_finite() {
            return Err(AlgorithmError::NonFiniteSample);
        }

        self.total += f64::from(sample) * f64::from(self.amplitude);
        self.weight += f64::from(self.amplitude.abs());

        if !self.total.is_finite() || !self.weight.is_finite() {
            return Err(AlgorithmError::InvalidAmplitude);
        }

        Ok(())
    }

    fn advance(&mut self, settings: FractalSettings) {
        self.frequency *= settings.lacunarity;
        self.amplitude *= settings.persistence;
    }

    fn finish(self, settings: FractalSettings) -> Result<f32, AlgorithmError> {
        let value = if settings.normalize {
            if self.weight == 0.0 {
                0.0
            } else {
                self.total / self.weight
            }
        } else {
            self.total
        };

        if !value.is_finite() || value.abs() > f64::from(f32::MAX) {
            return Err(AlgorithmError::InvalidAmplitude);
        }

        Ok(value as f32)
    }
}
