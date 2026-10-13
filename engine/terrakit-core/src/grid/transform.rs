//! Validated grid-to-world affine transforms.
//!
//! `GridTransform2` maps 2D grid coordinates into 3D world space, while
//! `GridTransform3` maps 3D grid coordinates into 3D world space.

use crate::{error::CoreError, math::Vector3F64};

/// A validated affine transform from 2D grid coordinates into world space.
///
/// The two axes must be finite, non-zero, and non-parallel. For heightmaps,
/// TerraKit uses the XZ world plane by default and stores elevation separately
/// as each height sample.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridTransform2 {
    origin: Vector3F64,
    axis_x: Vector3F64,
    axis_y: Vector3F64,
}

impl GridTransform2 {
    /// Identity transform for point samples on TerraKit's standard XZ plane.
    pub const fn identity_xz() -> Self {
        Self {
            origin: Vector3F64::ZERO,
            axis_x: Vector3F64::X,
            axis_y: Vector3F64::Z,
        }
    }

    /// Creates a standard XZ-plane transform with uniform positive `spacing`.
    pub fn from_spacing(spacing: f64) -> Result<Self, CoreError> {
        Self::from_axes(
            Vector3F64::ZERO,
            Vector3F64::new(spacing, 0.0, 0.0),
            Vector3F64::new(0.0, 0.0, spacing),
        )
    }

    /// Creates a 2D grid transform from `origin`, `axis_x`, and `axis_y`.
    ///
    /// Axes must be finite, non-zero, and non-parallel.
    pub fn from_axes(
        origin: Vector3F64,
        axis_x: Vector3F64,
        axis_y: Vector3F64,
    ) -> Result<Self, CoreError> {
        let transform = Self {
            origin,
            axis_x,
            axis_y,
        };

        if !transform.is_valid() {
            return Err(CoreError::InvalidTransform);
        }

        Ok(transform)
    }

    /// Returns the world-space position of grid coordinate `(0, 0)`.
    pub fn origin(self) -> Vector3F64 {
        self.origin
    }

    /// Returns the world-space step applied when grid X increases by one.
    pub fn axis_x(self) -> Vector3F64 {
        self.axis_x
    }

    /// Returns the world-space step applied when grid Y increases by one.
    pub fn axis_y(self) -> Vector3F64 {
        self.axis_y
    }

    /// Maps grid coordinate `x`, `y` into world space.
    pub fn position_at(self, x: f64, y: f64) -> Vector3F64 {
        Vector3F64::new(
            self.origin.x + self.axis_x.x * x + self.axis_y.x * y,
            self.origin.y + self.axis_x.y * x + self.axis_y.y * y,
            self.origin.z + self.axis_x.z * x + self.axis_y.z * y,
        )
    }

    /// Returns true when the origin and axes form a finite, non-degenerate basis.
    pub fn is_valid(self) -> bool {
        self.origin.is_finite()
            && self.axis_x.is_finite()
            && self.axis_y.is_finite()
            && self.axis_x.length_squared() > 0.0
            && self.axis_y.length_squared() > 0.0
            && self.axis_x.cross(self.axis_y).length_squared() > 0.0
    }
}

/// A validated affine transform from 3D grid coordinates into world space.
///
/// The three axes must be finite, non-zero, and form a non-degenerate basis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridTransform3 {
    origin: Vector3F64,
    axis_x: Vector3F64,
    axis_y: Vector3F64,
    axis_z: Vector3F64,
}

impl GridTransform3 {
    /// Identity transform for 3D grid coordinates.
    pub const fn identity() -> Self {
        Self {
            origin: Vector3F64::ZERO,
            axis_x: Vector3F64::X,
            axis_y: Vector3F64::Y,
            axis_z: Vector3F64::Z,
        }
    }

    /// Creates an axis-aligned 3D transform with uniform positive `spacing`.
    pub fn from_spacing(spacing: f64) -> Result<Self, CoreError> {
        Self::from_axes(
            Vector3F64::ZERO,
            Vector3F64::new(spacing, 0.0, 0.0),
            Vector3F64::new(0.0, spacing, 0.0),
            Vector3F64::new(0.0, 0.0, spacing),
        )
    }

    /// Creates a 3D grid transform from `origin`, `axis_x`, `axis_y`, and `axis_z`.
    ///
    /// Axes must be finite, non-zero, and form a non-degenerate 3D basis.
    pub fn from_axes(
        origin: Vector3F64,
        axis_x: Vector3F64,
        axis_y: Vector3F64,
        axis_z: Vector3F64,
    ) -> Result<Self, CoreError> {
        let transform = Self {
            origin,
            axis_x,
            axis_y,
            axis_z,
        };

        if !transform.is_valid() {
            return Err(CoreError::InvalidTransform);
        }

        Ok(transform)
    }

    /// Returns the world-space position of grid coordinate `(0, 0, 0)`.
    pub fn origin(self) -> Vector3F64 {
        self.origin
    }

    /// Returns the world-space step applied when grid X increases by one.
    pub fn axis_x(self) -> Vector3F64 {
        self.axis_x
    }

    /// Returns the world-space step applied when grid Y increases by one.
    pub fn axis_y(self) -> Vector3F64 {
        self.axis_y
    }

    /// Returns the world-space step applied when grid Z increases by one.
    pub fn axis_z(self) -> Vector3F64 {
        self.axis_z
    }

    /// Maps grid coordinate `x`, `y`, `z` into world space.
    pub fn position_at(self, x: f64, y: f64, z: f64) -> Vector3F64 {
        Vector3F64::new(
            self.origin.x + self.axis_x.x * x + self.axis_y.x * y + self.axis_z.x * z,
            self.origin.y + self.axis_x.y * x + self.axis_y.y * y + self.axis_z.y * z,
            self.origin.z + self.axis_x.z * x + self.axis_y.z * y + self.axis_z.z * z,
        )
    }

    /// Returns true when the origin and axes form a finite, non-degenerate basis.
    pub fn is_valid(self) -> bool {
        self.origin.is_finite()
            && self.axis_x.is_finite()
            && self.axis_y.is_finite()
            && self.axis_z.is_finite()
            && self.axis_x.length_squared() > 0.0
            && self.axis_y.length_squared() > 0.0
            && self.axis_z.length_squared() > 0.0
            && self.axis_x.cross(self.axis_y).dot(self.axis_z).abs() > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_transform2_from_spacing_uses_xz_ground_plane() {
        let transform = GridTransform2::from_spacing(2.0).unwrap();

        assert_eq!(transform.origin(), Vector3F64::ZERO);
        assert_eq!(transform.axis_x(), Vector3F64::new(2.0, 0.0, 0.0));
        assert_eq!(transform.axis_y(), Vector3F64::new(0.0, 0.0, 2.0));
        assert_eq!(
            transform.position_at(2.0, 3.0),
            Vector3F64::new(4.0, 0.0, 6.0)
        );
        assert!(transform.is_valid());
    }

    #[test]
    fn grid_transform2_rejects_zero_or_non_finite_axes() {
        let zero_spacing = GridTransform2::from_spacing(0.0);
        let non_finite_origin = GridTransform2::from_axes(
            Vector3F64::new(f64::NAN, 0.0, 0.0),
            Vector3F64::X,
            Vector3F64::Z,
        );

        assert_eq!(zero_spacing, Err(CoreError::InvalidTransform));
        assert_eq!(non_finite_origin, Err(CoreError::InvalidTransform));
    }

    #[test]
    fn grid_transform2_rejects_parallel_axes() {
        let transform = GridTransform2::from_axes(Vector3F64::ZERO, Vector3F64::X, Vector3F64::X);

        assert_eq!(transform, Err(CoreError::InvalidTransform));
    }

    #[test]
    fn grid_transform3_from_spacing_uses_xyz_axes() {
        let transform = GridTransform3::from_spacing(3.0).unwrap();

        assert_eq!(transform.axis_x(), Vector3F64::new(3.0, 0.0, 0.0));
        assert_eq!(transform.axis_y(), Vector3F64::new(0.0, 3.0, 0.0));
        assert_eq!(transform.axis_z(), Vector3F64::new(0.0, 0.0, 3.0));
        assert_eq!(
            transform.position_at(1.0, 2.0, 3.0),
            Vector3F64::new(3.0, 6.0, 9.0)
        );
        assert!(transform.is_valid());
    }

    #[test]
    fn grid_transform3_rejects_coplanar_axes() {
        let transform = GridTransform3::from_axes(
            Vector3F64::ZERO,
            Vector3F64::X,
            Vector3F64::Y,
            Vector3F64::new(1.0, 1.0, 0.0),
        );

        assert_eq!(transform, Err(CoreError::InvalidTransform));
    }
}
