//! Shared algorithm-local scattering types and validation.

/// Scatter options shared by uniform sampling algorithms.
#[derive(Debug, Clone, Copy)]
pub struct ScatterSettings {
    /// Deterministic random seed.
    pub seed: u64,
    /// Prototype identifier to assign to every placement.
    pub prototype_id: u32,
    /// Minimum uniform scale.
    pub min_scale: f32,
    /// Maximum uniform scale.
    pub max_scale: f32,
    /// Randomize each placement's yaw.
    pub random_rotation: bool,
}

impl Default for ScatterSettings {
    fn default() -> Self {
        Self {
            seed: 0,
            prototype_id: 0,
            min_scale: 1.0,
            max_scale: 1.0,
            random_rotation: true,
        }
    }
}

impl ScatterSettings {
    /// Validates a scale range and finite options.
    pub fn validate(&self) -> Result<(), ScatterError> {
        if !self.min_scale.is_finite()
            || !self.max_scale.is_finite()
            || self.min_scale < 0.0
            || self.max_scale < self.min_scale
        {
            return Err(ScatterError::InvalidSettings("invalid scale range"));
        }
        Ok(())
    }
}

/// Axis-aligned XZ sampling bounds (in algorithm-local coordinates).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScatterArea {
    /// Minimum X coordinate.
    pub min_x: f32,
    /// Maximum X coordinate.
    pub max_x: f32,
    /// Minimum Z coordinate.
    pub min_z: f32,
    /// Maximum Z coordinate.
    pub max_z: f32,
}

impl ScatterArea {
    /// Creates a rectangular scattering area.
    pub fn new(min_x: f32, max_x: f32, min_z: f32, max_z: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_z,
            max_z,
        }
    }

    /// Returns the rectangle's X span.
    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    /// Returns the rectangle's Z span.
    pub fn depth(&self) -> f32 {
        self.max_z - self.min_z
    }

    /// Returns the rectangle's XZ area.
    pub fn area(&self) -> f32 {
        self.width().abs() * self.depth().abs()
    }

    /// Checks the area has finite, strictly ordered bounds and finite spans.
    pub fn validate(&self) -> Result<(), ScatterError> {
        if !self.min_x.is_finite()
            || !self.max_x.is_finite()
            || !self.min_z.is_finite()
            || !self.max_z.is_finite()
            || self.max_x <= self.min_x
            || self.max_z <= self.min_z
            || !self.width().is_finite()
            || !self.depth().is_finite()
        {
            return Err(ScatterError::InvalidArea);
        }
        Ok(())
    }

    /// Converts normalized XZ coordinates into local position coordinates.
    pub fn point_from_normalized(&self, x: f32, z: f32) -> [f32; 3] {
        [
            self.min_x + x * self.width(),
            0.0,
            self.min_z + z * self.depth(),
        ]
    }
}

/// Terrain surface position and normal in local space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TerrainSample {
    /// Sampled surface position.
    pub position: [f32; 3],
    /// Sampled surface normal.
    pub normal: [f32; 3],
}

impl TerrainSample {
    /// Creates a terrain sample.
    pub fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self { position, normal }
    }

    /// Returns slope degrees relative to the positive Y axis.
    pub fn slope_degrees(&self) -> f32 {
        let normal_length = (self.normal[0] * self.normal[0]
            + self.normal[1] * self.normal[1]
            + self.normal[2] * self.normal[2])
            .sqrt();
        if normal_length <= f32::EPSILON {
            return 90.0;
        }
        (self.normal[1] / normal_length)
            .clamp(-1.0, 1.0)
            .acos()
            .to_degrees()
    }
}

/// Validation and resource-budget errors from scattering algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScatterError {
    /// A setting is incompatible or invalid.
    InvalidSettings(&'static str),
    /// A sampling area is non-finite or degenerate.
    InvalidArea,
    /// A Poisson separation radius is invalid.
    InvalidRadius,
    /// No candidate attempts were requested.
    InvalidAttempts,
    /// A mask was configured incorrectly.
    InvalidMask,
    /// The Poisson occupancy grid would require excessive memory.
    GridTooLarge,
    /// Sampling exceeded a bounded CPU-work budget.
    BudgetExceeded,
}
