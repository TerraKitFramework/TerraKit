//! Masks used to filter generated object placements.

use super::types::{ScatterError, TerrainSample};

/// Determines whether an object can be placed at a location.
pub trait ScatterMask {
    /// Returns true if the location is allowed.
    fn allows(&self, position: [f32; 3]) -> bool;
}

/// Allows objects only within a specific height range.
#[derive(Debug, Clone, Copy)]
pub struct HeightMask {
    /// Minimum height.
    pub min_height: f32,
    /// Maximum height.
    pub max_height: f32,
}

impl HeightMask {
    /// Creates a new height mask.
    pub fn new(min_height: f32, max_height: f32) -> Result<Self, ScatterError> {
        if !min_height.is_finite() || !max_height.is_finite() || max_height < min_height {
            return Err(ScatterError::InvalidMask);
        }

        Ok(Self {
            min_height,
            max_height,
        })
    }

    /// Tests a height value directly.
    pub fn allows_height(&self, height: f32) -> bool {
        height >= self.min_height && height <= self.max_height
    }
}

impl ScatterMask for HeightMask {
    fn allows(&self, position: [f32; 3]) -> bool {
        self.allows_height(position[1])
    }
}

/// Allows objects only on surfaces within a slope range.
#[derive(Debug, Clone, Copy)]
pub struct SlopeMask {
    /// Minimum slope in degrees.
    pub min_slope: f32,
    /// Maximum slope in degrees.
    pub max_slope: f32,
}

impl SlopeMask {
    /// Creates a slope mask.
    pub fn new(min_slope: f32, max_slope: f32) -> Result<Self, ScatterError> {
        if !min_slope.is_finite()
            || !max_slope.is_finite()
            || min_slope < 0.0
            || max_slope > 90.0
            || max_slope < min_slope
        {
            return Err(ScatterError::InvalidMask);
        }

        Ok(Self {
            min_slope,
            max_slope,
        })
    }

    /// Tests a terrain sample.
    pub fn allows_sample(&self, sample: &TerrainSample) -> bool {
        let slope = sample.slope_degrees();

        slope >= self.min_slope && slope <= self.max_slope
    }
}

/// Allows objects according to a density value.
#[derive(Debug, Clone, Copy)]
pub struct DensityMask {
    /// Minimum density required.
    pub threshold: f32,
}

impl DensityMask {
    /// Creates a density mask.
    pub fn new(threshold: f32) -> Result<Self, ScatterError> {
        if !threshold.is_finite() {
            return Err(ScatterError::InvalidMask);
        }

        Ok(Self { threshold })
    }

    /// Tests a density value.
    pub fn allows_density(&self, density: f32) -> bool {
        density >= self.threshold
    }
}

/// Combines multiple masks using logical AND.
#[derive(Debug, Clone, Copy)]
pub struct CombinedMask<M1, M2> {
    /// First required mask.
    pub first: M1,
    /// Second required mask.
    pub second: M2,
}

impl<M1, M2> CombinedMask<M1, M2> {
    /// Creates an AND mask.
    pub fn new(first: M1, second: M2) -> Self {
        Self { first, second }
    }
}

impl<M1, M2> ScatterMask for CombinedMask<M1, M2>
where
    M1: ScatterMask,
    M2: ScatterMask,
{
    fn allows(&self, position: [f32; 3]) -> bool {
        self.first.allows(position) && self.second.allows(position)
    }
}

/// Combines two masks using logical OR.
#[derive(Debug, Clone, Copy)]
pub struct AnyMask<M1, M2> {
    /// First alternative mask.
    pub first: M1,
    /// Second alternative mask.
    pub second: M2,
}

impl<M1, M2> AnyMask<M1, M2> {
    /// Creates an OR mask.
    pub fn new(first: M1, second: M2) -> Self {
        Self { first, second }
    }
}

impl<M1, M2> ScatterMask for AnyMask<M1, M2>
where
    M1: ScatterMask,
    M2: ScatterMask,
{
    fn allows(&self, position: [f32; 3]) -> bool {
        self.first.allows(position) || self.second.allows(position)
    }
}

/// Negates another mask.
#[derive(Debug, Clone, Copy)]
pub struct NotMask<M> {
    /// Mask whose result is inverted.
    pub mask: M,
}

impl<M> NotMask<M> {
    /// Creates a NOT mask.
    pub fn new(mask: M) -> Self {
        Self { mask }
    }
}

impl<M> ScatterMask for NotMask<M>
where
    M: ScatterMask,
{
    fn allows(&self, position: [f32; 3]) -> bool {
        !self.mask.allows(position)
    }
}
