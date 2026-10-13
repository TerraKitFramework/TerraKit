//! Engine-independent object placement resources.

use crate::Vector3F64;

/// One generated placement. The position is local to its collection's origin.
///
/// The layout is shared with the borrowed C ABI scatter-point view. Do not
/// reorder or change fields without an ABI compatibility review.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScatterPoint {
    /// Position relative to `ScatterPoints::origin()`.
    pub position: [f32; 3],
    /// Quaternion rotation in XYZW order.
    pub rotation: [f32; 4],
    /// Non-uniform scale in XYZ order.
    pub scale: [f32; 3],
    /// Prototype identifier whose interpretation belongs to the host.
    pub prototype_id: u32,
}

/// Origin-relative placement collection with precise world coordinates.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScatterPoints {
    origin: Vector3F64,
    points: Vec<ScatterPoint>,
}

impl ScatterPoints {
    /// Creates an empty collection at the world origin.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates points relative to the world origin.
    pub fn from_vec(points: Vec<ScatterPoint>) -> Self {
        Self::with_origin(Vector3F64::ZERO, points)
    }

    /// Creates points whose positions are relative to the given world origin.
    pub fn with_origin(origin: Vector3F64, points: Vec<ScatterPoint>) -> Self {
        Self { origin, points }
    }

    /// Returns the precise world-space origin of the collection.
    pub fn origin(&self) -> Vector3F64 {
        self.origin
    }

    /// Returns the local scatter placements.
    pub fn points(&self) -> &[ScatterPoint] {
        &self.points
    }

    /// Returns the number of placements.
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Returns whether the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Consumes the collection, returning only the local placements.
    pub fn into_vec(self) -> Vec<ScatterPoint> {
        self.points
    }
}
