//! Vector types used by TerraKit coordinates, transforms, and mesh attributes.

use std::ops::{Add, AddAssign, Mul, Sub};

/// A compact two-component vector stored as 32-bit floats.
///
/// This type is suitable as an element in immutable zero-copy foreign-function
/// buffer views. Its fields must not be reordered or changed without treating
/// that as a compatibility-breaking change.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector2F32 {
    /// X component.
    pub x: f32,

    /// Y component.
    pub y: f32,
}

impl Vector2F32 {
    /// A vector with both components set to zero.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Creates a vector from `x` and `y` components.
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Returns true when both components are finite numbers.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Returns the squared Euclidean length.
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y
    }
}

/// A precise two-component vector stored as 64-bit floats.
///
/// This type is suitable as an element in immutable zero-copy foreign-function
/// buffer views. Its fields must not be reordered or changed without treating
/// that as a compatibility-breaking change.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector2F64 {
    /// X component.
    pub x: f64,

    /// Y component.
    pub y: f64,
}

impl Vector2F64 {
    /// A vector with both components set to zero.
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// Creates a vector from explicit components.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Returns true when both components are finite numbers.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Returns the squared Euclidean length.
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }
}

/// A compact three-component vector stored as 32-bit floats.
///
/// This type is suitable as an element in immutable zero-copy foreign-function
/// buffer views. Its fields must not be reordered or changed without treating
/// that as a compatibility-breaking change.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector3F32 {
    /// X component.
    pub x: f32,

    /// Y component.
    pub y: f32,

    /// Z component.
    pub z: f32,
}

impl Vector3F32 {
    /// A vector with all components set to zero.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Positive unit vector along the X axis.
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };

    /// Positive unit vector along the Y axis.
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    /// Positive unit vector along the Z axis.
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    /// Creates a vector from `x`, `y`, and `z` components.
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Returns true when all components are finite numbers.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Returns the squared Euclidean length.
    pub fn length_squared(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the Euclidean length.
    pub fn length(self) -> f32 {
        self.length_squared().sqrt()
    }

    /// Returns the dot product with `rhs`.
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// Returns the cross product with `rhs`.
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// Returns a unit-length vector, or `None` for zero or non-finite length.
    pub fn normalized(self) -> Option<Self> {
        let length = self.length();

        if !length.is_finite() || length == 0.0 {
            return None;
        }

        Some(Self {
            x: self.x / length,
            y: self.y / length,
            z: self.z / length,
        })
    }
}

impl Add for Vector3F32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl AddAssign for Vector3F32 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Sub for Vector3F32 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for Vector3F32 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

/// A precise three-component vector stored as 64-bit floats.
///
/// This type is suitable as an element in immutable zero-copy foreign-function
/// buffer views. Its fields must not be reordered or changed without treating
/// that as a compatibility-breaking change.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector3F64 {
    /// X component.
    pub x: f64,

    /// Y component.
    pub y: f64,

    /// Z component.
    pub z: f64,
}

impl Vector3F64 {
    /// A vector with all components set to zero.
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    /// Positive unit vector along the X axis.
    pub const X: Self = Self {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };

    /// Positive unit vector along the Y axis.
    pub const Y: Self = Self {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };

    /// Positive unit vector along the Z axis.
    pub const Z: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    /// Creates a vector from `x`, `y`, and `z` components.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Returns true when all components are finite numbers.
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Returns the squared Euclidean length.
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Returns the dot product with `rhs`.
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// Returns the cross product with `rhs`.
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector2_f32_new_sets_components() {
        let vector = Vector2F32::new(1.0, 2.0);

        assert_eq!(vector.x, 1.0);
        assert_eq!(vector.y, 2.0);
    }

    #[test]
    fn vector2_f64_new_sets_components_and_detects_non_finite_values() {
        let vector = Vector2F64::new(1.0, 2.0);

        assert_eq!(vector.x, 1.0);
        assert_eq!(vector.y, 2.0);
        assert!(vector.is_finite());
        assert!(!Vector2F64::new(f64::INFINITY, 1.0).is_finite());
    }

    #[test]
    fn vector3_f32_detects_non_finite_components() {
        assert!(Vector3F32::new(1.0, 2.0, 3.0).is_finite());
        assert!(!Vector3F32::new(1.0, f32::INFINITY, 3.0).is_finite());
    }

    #[test]
    fn vector3_f32_supports_basic_vector_math() {
        let lhs = Vector3F32::new(1.0, 2.0, 3.0);
        let rhs = Vector3F32::new(4.0, 5.0, 6.0);

        assert_eq!(lhs + rhs, Vector3F32::new(5.0, 7.0, 9.0));
        assert_eq!(rhs - lhs, Vector3F32::new(3.0, 3.0, 3.0));
        assert_eq!(lhs * 2.0, Vector3F32::new(2.0, 4.0, 6.0));
        assert_eq!(Vector3F32::X.dot(Vector3F32::Y), 0.0);
        assert_eq!(Vector3F32::X.cross(Vector3F32::Y), Vector3F32::Z);
    }

    #[test]
    fn vector3_f32_normalized_handles_zero_without_nan() {
        assert_eq!(Vector3F32::ZERO.normalized(), None);
        assert_eq!(
            Vector3F32::new(0.0, 3.0, 0.0).normalized(),
            Some(Vector3F32::Y)
        );
    }

    #[test]
    fn vector3_f64_exposes_axis_constants() {
        assert_eq!(Vector3F64::ZERO, Vector3F64::new(0.0, 0.0, 0.0));
        assert_eq!(Vector3F64::X, Vector3F64::new(1.0, 0.0, 0.0));
        assert_eq!(Vector3F64::Y, Vector3F64::new(0.0, 1.0, 0.0));
        assert_eq!(Vector3F64::Z, Vector3F64::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn vector3_f64_length_squared_uses_all_components() {
        assert_eq!(Vector3F64::new(2.0, 3.0, 6.0).length_squared(), 49.0);
    }

    #[test]
    fn vector3_f64_dot_and_cross_cover_basic_transform_math() {
        assert_eq!(Vector3F64::X.dot(Vector3F64::Y), 0.0);
        assert_eq!(Vector3F64::X.cross(Vector3F64::Y), Vector3F64::Z);
    }
}
