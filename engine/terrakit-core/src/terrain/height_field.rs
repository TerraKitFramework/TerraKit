//! Height-field terrain resource.
//!
//! A height field wraps a 2D spatial field of finite samples and a world-space
//! height axis used to lift each sample away from the base field plane.

use crate::{
    error::CoreError,
    grid::{Extent2, Field2, GridTransform2, SamplingDomain},
    math::Vector3F64,
};

/// A single terrain elevation sample.
pub type HeightSample = f32;

/// A two-dimensional terrain surface represented by finite height samples.
///
/// The underlying `Field2` defines the base sampling plane. `height_axis`
/// defines where each height sample is applied from that plane. TerraKit's
/// standard convention is XZ as the horizontal plane and `Vector3F64::Y` as up.
/// Values are stored as contiguous `f32` samples in X-contiguous row order:
/// `index = y * width + x`.
#[derive(Debug, Clone, PartialEq)]
pub struct HeightField {
    field: Field2<HeightSample>,
    height_axis: Vector3F64,
}

impl HeightField {
    /// Creates a point-sampled, zero-filled height field using canonical shapes.
    pub fn new(
        extent: Extent2,
        transform: GridTransform2,
        height_axis: Vector3F64,
    ) -> Result<Self, CoreError> {
        Self::filled(extent, 0.0, transform, SamplingDomain::Points, height_axis)
    }

    /// Convenience constructor for prototyping when dimensions start as `usize`.
    pub fn from_dimensions(width: usize, height: usize) -> Result<Self, CoreError> {
        let extent = Extent2::from_usize(width, height)?;

        Self::new(extent, GridTransform2::identity_xz(), Vector3F64::Y)
    }

    /// Creates a height field of `extent` filled with `value`.
    ///
    /// `transform` defines the base sample plane, `sampling` defines whether
    /// samples live on points or cells, and `height_axis` defines the direction
    /// and scale used when applying height values.
    pub fn filled(
        extent: Extent2,
        value: HeightSample,
        transform: GridTransform2,
        sampling: SamplingDomain,
        height_axis: Vector3F64,
    ) -> Result<Self, CoreError> {
        validate_height_sample(value)?;
        validate_height_axis(height_axis)?;

        let field = Field2::filled(extent, value, transform, sampling)?;

        Ok(Self { field, height_axis })
    }

    /// Creates a height field from explicit `values`.
    ///
    /// `values` must match `extent.cell_count()`, all samples must be finite,
    /// `transform` places base samples in world space, `sampling` identifies
    /// the sample domain, and `height_axis` must be finite and non-zero.
    pub fn from_values(
        extent: Extent2,
        values: Vec<HeightSample>,
        transform: GridTransform2,
        sampling: SamplingDomain,
        height_axis: Vector3F64,
    ) -> Result<Self, CoreError> {
        validate_height_samples(&values)?;
        validate_height_axis(height_axis)?;

        let field = Field2::from_vec(extent, values, transform, sampling)?;

        Ok(Self { field, height_axis })
    }

    /// Creates a height field from an existing spatial `field` and `height_axis`.
    ///
    /// All field values must be finite and `height_axis` must be finite and
    /// non-zero.
    pub fn from_field(
        field: Field2<HeightSample>,
        height_axis: Vector3F64,
    ) -> Result<Self, CoreError> {
        validate_height_samples(field.values())?;
        validate_height_axis(height_axis)?;

        Ok(Self { field, height_axis })
    }

    /// Returns the validated 2D extent of the height samples.
    pub fn extent(&self) -> Extent2 {
        self.field.extent()
    }

    /// Returns the height-field width as `usize`.
    pub fn width(&self) -> usize {
        self.field.width()
    }

    /// Returns the height-field height as `usize`.
    pub fn height(&self) -> usize {
        self.field.height()
    }

    /// Returns the number of height samples.
    pub fn len(&self) -> usize {
        self.field.len()
    }

    /// Returns true when the height field stores no samples.
    ///
    /// Valid extents are non-empty, so this is normally false for constructed
    /// height fields.
    pub fn is_empty(&self) -> bool {
        self.field.is_empty()
    }

    /// Returns all finite height samples in X-contiguous row order.
    ///
    /// The storage index is `y * width + x`. When accessed through a future
    /// foreign-function view, this slice is borrowed from the owning generation
    /// result.
    pub fn values(&self) -> &[HeightSample] {
        self.field.values()
    }

    /// Returns the base grid-to-world transform.
    pub fn transform(&self) -> GridTransform2 {
        self.field.transform()
    }

    /// Returns whether height values are point samples or cell samples.
    pub fn sampling(&self) -> SamplingDomain {
        self.field.sampling()
    }

    /// Returns the world-space axis used to apply height samples.
    pub fn height_axis(&self) -> Vector3F64 {
        self.height_axis
    }

    /// Returns the underlying typed spatial field.
    pub fn field(&self) -> &Field2<HeightSample> {
        &self.field
    }

    /// Returns the sample at `x`, `y`, or `None` when out of bounds.
    pub fn get(&self, x: usize, y: usize) -> Option<HeightSample> {
        self.field.get_usize(x, y).copied()
    }

    /// Writes finite `value` at `x`, `y`, returning a validation or bounds error on failure.
    pub fn try_set(&mut self, x: usize, y: usize, value: HeightSample) -> Result<(), CoreError> {
        validate_height_sample(value)?;

        self.field.try_set(x, y, value)
    }

    /// Returns the world-space position on the base field plane before height is applied.
    pub fn base_position(&self, x: usize, y: usize) -> Option<Vector3F64> {
        self.field.sample_position(x, y)
    }

    /// Returns the final world-space terrain surface position for a sample.
    pub fn surface_position(&self, x: usize, y: usize) -> Option<Vector3F64> {
        let base = self.base_position(x, y)?;
        let height = f64::from(self.get(x, y)?);

        Some(Vector3F64::new(
            base.x + self.height_axis.x * height,
            base.y + self.height_axis.y * height,
            base.z + self.height_axis.z * height,
        ))
    }

    /// Consumes the height field and returns its underlying spatial field.
    pub fn into_field(self) -> Field2<HeightSample> {
        self.field
    }
}

fn validate_height_sample(value: HeightSample) -> Result<(), CoreError> {
    if !value.is_finite() {
        return Err(CoreError::NonFiniteHeightSample);
    }

    Ok(())
}

fn validate_height_samples(values: &[HeightSample]) -> Result<(), CoreError> {
    if values.iter().any(|value| !value.is_finite()) {
        return Err(CoreError::NonFiniteHeightSample);
    }

    Ok(())
}

fn validate_height_axis(axis: Vector3F64) -> Result<(), CoreError> {
    if !axis.is_finite() || axis.length_squared() == 0.0 {
        return Err(CoreError::InvalidHeightAxis);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard_extent() -> Extent2 {
        Extent2::try_new(3, 2).unwrap()
    }

    #[test]
    fn height_field_stores_values_in_x_contiguous_order() {
        let mut field = HeightField::new(
            standard_extent(),
            GridTransform2::identity_xz(),
            Vector3F64::Y,
        )
        .unwrap();

        field.try_set(2, 1, 4.25).unwrap();

        assert_eq!(field.get(2, 1), Some(4.25));
        assert_eq!(field.values(), &[0.0, 0.0, 0.0, 0.0, 0.0, 4.25]);
    }

    #[test]
    fn height_field_rejects_invalid_samples() {
        let error = HeightField::filled(
            standard_extent(),
            f32::NAN,
            GridTransform2::identity_xz(),
            SamplingDomain::Points,
            Vector3F64::Y,
        )
        .unwrap_err();

        assert_eq!(error, CoreError::NonFiniteHeightSample);
    }

    #[test]
    fn height_field_rejects_invalid_height_axes() {
        let error = HeightField::new(
            standard_extent(),
            GridTransform2::identity_xz(),
            Vector3F64::ZERO,
        )
        .unwrap_err();

        assert_eq!(error, CoreError::InvalidHeightAxis);
    }

    #[test]
    fn height_field_reports_out_of_bounds_writes() {
        let mut field = HeightField::new(
            standard_extent(),
            GridTransform2::identity_xz(),
            Vector3F64::Y,
        )
        .unwrap();

        assert_eq!(
            field.try_set(3, 0, 1.0),
            Err(CoreError::CoordinateOutOfBounds2 {
                x: 3,
                y: 0,
                width: 3,
                height: 2,
            })
        );
    }

    #[test]
    fn height_field_surface_position_applies_height_axis() {
        let transform = GridTransform2::from_spacing(2.0).unwrap();
        let mut field = HeightField::new(standard_extent(), transform, Vector3F64::Y).unwrap();

        field.try_set(1, 1, 3.0).unwrap();

        assert_eq!(
            field.base_position(1, 1),
            Some(Vector3F64::new(2.0, 0.0, 2.0))
        );
        assert_eq!(
            field.surface_position(1, 1),
            Some(Vector3F64::new(2.0, 3.0, 2.0))
        );
    }
}
