//! Density-field terrain resource.
//!
//! A density field wraps a 3D spatial field of finite scalar samples for
//! implicit terrain and signed-density workflows.

use crate::{
    error::CoreError,
    grid::{Extent3, Field3, GridTransform3, SamplingDomain},
};

/// A single scalar density sample.
pub type DensitySample = f32;

/// A finite 3D scalar field for implicit terrain or signed-density workflows.
///
/// Values are stored as contiguous `f32` samples in X-contiguous layer order:
/// `index = (z * height + y) * width + x`.
#[derive(Debug, Clone, PartialEq)]
pub struct DensityField {
    field: Field3<DensitySample>,
}

impl DensityField {
    /// Creates a zero-filled density field of `extent`.
    ///
    /// `transform` places grid coordinates in world space and `sampling`
    /// defines whether samples live on points or cells.
    pub fn new(
        extent: Extent3,
        transform: GridTransform3,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        Self::filled(extent, 0.0, transform, sampling)
    }

    /// Creates a point-sampled zero density field from `width`, `height`, and `depth`.
    pub fn from_dimensions(width: usize, height: usize, depth: usize) -> Result<Self, CoreError> {
        let extent = Extent3::from_usize(width, height, depth)?;

        Self::new(extent, GridTransform3::identity(), SamplingDomain::Points)
    }

    /// Creates a density field of `extent` filled with finite `value`.
    ///
    /// `transform` places samples in world space and `sampling` declares the
    /// point or cell convention.
    pub fn filled(
        extent: Extent3,
        value: DensitySample,
        transform: GridTransform3,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        validate_density_sample(value)?;

        let field = Field3::filled(extent, value, transform, sampling)?;

        Ok(Self { field })
    }

    /// Creates a density field from explicit finite `values`.
    ///
    /// `values` must match `extent.cell_count()`, `transform` places samples in
    /// world space, and `sampling` declares the point or cell convention.
    pub fn from_values(
        extent: Extent3,
        values: Vec<DensitySample>,
        transform: GridTransform3,
        sampling: SamplingDomain,
    ) -> Result<Self, CoreError> {
        validate_density_samples(&values)?;

        let field = Field3::from_vec(extent, values, transform, sampling)?;

        Ok(Self { field })
    }

    /// Creates a density field from an existing spatial `field`.
    ///
    /// Every sample in `field` must be finite.
    pub fn from_field(field: Field3<DensitySample>) -> Result<Self, CoreError> {
        validate_density_samples(field.values())?;

        Ok(Self { field })
    }

    /// Returns the validated 3D extent of the density samples.
    pub fn extent(&self) -> Extent3 {
        self.field.extent()
    }

    /// Returns the density-field width as `usize`.
    pub fn width(&self) -> usize {
        self.field.width()
    }

    /// Returns the density-field height as `usize`.
    pub fn height(&self) -> usize {
        self.field.height()
    }

    /// Returns the density-field depth as `usize`.
    pub fn depth(&self) -> usize {
        self.field.depth()
    }

    /// Returns the number of density samples.
    pub fn len(&self) -> usize {
        self.field.len()
    }

    /// Returns true when the density field stores no samples.
    ///
    /// Valid extents are non-empty, so this is normally false for constructed
    /// density fields.
    pub fn is_empty(&self) -> bool {
        self.field.is_empty()
    }

    /// Returns all finite density samples in X-contiguous layer order.
    ///
    /// The storage index is `(z * height + y) * width + x`. When accessed
    /// through a future foreign-function view, this slice is borrowed from the
    /// owning generation result.
    pub fn values(&self) -> &[DensitySample] {
        self.field.values()
    }

    /// Returns the underlying typed spatial field.
    pub fn field(&self) -> &Field3<DensitySample> {
        &self.field
    }

    /// Returns the sample at `x`, `y`, `z`, or `None` when out of bounds.
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<DensitySample> {
        self.field.get_usize(x, y, z).copied()
    }

    /// Writes finite `value` at `x`, `y`, `z`, returning a validation or bounds error on failure.
    pub fn try_set(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        value: DensitySample,
    ) -> Result<(), CoreError> {
        validate_density_sample(value)?;

        self.field.try_set(x, y, z, value)
    }

    /// Consumes the density field and returns its underlying spatial field.
    pub fn into_field(self) -> Field3<DensitySample> {
        self.field
    }
}

fn validate_density_sample(value: DensitySample) -> Result<(), CoreError> {
    if !value.is_finite() {
        return Err(CoreError::NonFiniteDensitySample);
    }

    Ok(())
}

fn validate_density_samples(values: &[DensitySample]) -> Result<(), CoreError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(CoreError::NonFiniteDensitySample);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_field_rejects_non_finite_samples() {
        let extent = Extent3::try_new(2, 2, 2).unwrap();
        let error = DensityField::filled(
            extent,
            f32::INFINITY,
            GridTransform3::identity(),
            SamplingDomain::Points,
        )
        .unwrap_err();

        assert_eq!(error, CoreError::NonFiniteDensitySample);
    }

    #[test]
    fn density_field_try_set_reports_bounds_errors() {
        let mut field = DensityField::from_dimensions(2, 2, 2).unwrap();

        assert_eq!(
            field.try_set(0, 0, 2, 1.0),
            Err(CoreError::CoordinateOutOfBounds3 {
                x: 0,
                y: 0,
                z: 2,
                width: 2,
                height: 2,
                depth: 2,
            })
        );
    }

    #[test]
    fn density_field_stores_values() {
        let mut field = DensityField::from_dimensions(2, 2, 2).unwrap();

        field.try_set(1, 1, 1, -0.5).unwrap();

        assert_eq!(field.get(1, 1, 1), Some(-0.5));
    }
}
