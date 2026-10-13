//! Built-in deterministic noise algorithm selectors.

use terrakit_core::GenerationSeed;

use super::{
    Noise2D, Noise3D, PerlinNoise2D, PerlinNoise3D, SimplexNoise2D, SimplexNoise3D, ValueNoise2D,
    ValueNoise3D, WorleyNoise2D, WorleyNoise3D,
};

/// First-party 2D noise implementations shipped by TerraKit.
///
/// This enum is for built-in deterministic algorithms only. It does not model
/// external plugins or dynamic algorithm loading.
#[derive(Debug, Clone)]
pub enum BuiltinNoise2D {
    /// Coherent value noise.
    Value(ValueNoise2D),

    /// Coherent gradient noise using Perlin-style lattice gradients.
    Perlin(PerlinNoise2D),

    /// Coherent gradient noise using a simplex lattice.
    Simplex(SimplexNoise2D),

    /// Signed cellular distance noise.
    Worley(WorleyNoise2D),
}

impl Noise2D for BuiltinNoise2D {
    fn sample(&self, x: f64, y: f64) -> f32 {
        match self {
            Self::Value(noise) => noise.sample(x, y),
            Self::Perlin(noise) => noise.sample(x, y),
            Self::Simplex(noise) => noise.sample(x, y),
            Self::Worley(noise) => noise.sample(x, y),
        }
    }
}

/// First-party 3D noise implementations shipped by TerraKit.
///
/// This enum is for built-in deterministic algorithms only. It does not model
/// external plugins or dynamic algorithm loading.
#[derive(Debug, Clone)]
pub enum BuiltinNoise3D {
    /// Coherent value noise.
    Value(ValueNoise3D),

    /// Coherent gradient noise using Perlin-style lattice gradients.
    Perlin(PerlinNoise3D),

    /// Coherent gradient noise using a simplex lattice.
    Simplex(SimplexNoise3D),

    /// Signed cellular distance noise.
    Worley(WorleyNoise3D),
}

impl Noise3D for BuiltinNoise3D {
    fn sample(&self, x: f64, y: f64, z: f64) -> f32 {
        match self {
            Self::Value(noise) => noise.sample(x, y, z),
            Self::Perlin(noise) => noise.sample(x, y, z),
            Self::Simplex(noise) => noise.sample(x, y, z),
            Self::Worley(noise) => noise.sample(x, y, z),
        }
    }
}

/// Identifier for first-party noise algorithms available in TerraKit.
///
/// This enum is configuration-friendly and intentionally covers only built-in
/// algorithms. External plugins should use their own identifiers outside this
/// crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoiseAlgorithm {
    /// Coherent value noise.
    Value,

    /// Coherent gradient noise using Perlin-style lattice gradients.
    Perlin,

    /// Coherent gradient noise using a simplex lattice.
    Simplex,

    /// Signed cellular distance noise.
    Worley,
}

impl NoiseAlgorithm {
    /// Creates a 2D built-in noise sampler using the supplied deterministic
    /// generation seed.
    pub fn create_2d(self, seed: GenerationSeed) -> BuiltinNoise2D {
        match self {
            Self::Value => BuiltinNoise2D::Value(ValueNoise2D::new(seed)),
            Self::Perlin => BuiltinNoise2D::Perlin(PerlinNoise2D::new(seed)),
            Self::Simplex => BuiltinNoise2D::Simplex(SimplexNoise2D::new(seed)),
            Self::Worley => BuiltinNoise2D::Worley(WorleyNoise2D::new(seed)),
        }
    }

    /// Creates a 3D built-in noise sampler using the supplied deterministic
    /// generation seed.
    pub fn create_3d(self, seed: GenerationSeed) -> BuiltinNoise3D {
        match self {
            Self::Value => BuiltinNoise3D::Value(ValueNoise3D::new(seed)),
            Self::Perlin => BuiltinNoise3D::Perlin(PerlinNoise3D::new(seed)),
            Self::Simplex => BuiltinNoise3D::Simplex(SimplexNoise3D::new(seed)),
            Self::Worley => BuiltinNoise3D::Worley(WorleyNoise3D::new(seed)),
        }
    }
}
