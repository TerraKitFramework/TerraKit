//! Resource keys, resource kinds, and per-execution resource storage.

use std::collections::HashMap;

use terrakit_core::{
    DensityField, HeightField, ScatterPoints, TerrainMesh, TerrainResource, VoxelVolume,
};

/// Stable identifier for a resource within one pipeline execution.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceKey(
    /// Raw stable resource key.
    pub u64,
);

impl ResourceKey {
    /// Conventional key for the primary height field.
    pub const HEIGHT: Self = Self(1);

    /// Conventional key for the primary density field.
    pub const DENSITY: Self = Self(2);

    /// Conventional key for the primary voxel volume.
    pub const VOXELS: Self = Self(3);

    /// Conventional key for the primary mesh.
    pub const MESH: Self = Self(4);

    /// Conventional key for primary scatter points.
    pub const SCATTER_POINTS: Self = Self(5);
}

/// Pipeline-level classification for canonical terrain resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    /// A `terrakit_core::HeightField`.
    HeightField,

    /// A `terrakit_core::DensityField`.
    DensityField,

    /// A `terrakit_core::VoxelVolume`.
    VoxelVolume,

    /// A `terrakit_core::TerrainMesh`.
    Mesh,

    /// A [`terrakit_core::ScatterPoints`] collection.
    ScatterPoints,
}

/// Returns the pipeline resource kind for a canonical TerraKit resource.
pub fn resource_kind(resource: &TerrainResource) -> ResourceKind {
    match resource {
        TerrainResource::HeightField(_) => ResourceKind::HeightField,
        TerrainResource::DensityField(_) => ResourceKind::DensityField,
        TerrainResource::VoxelVolume(_) => ResourceKind::VoxelVolume,
        TerrainResource::Mesh(_) => ResourceKind::Mesh,
        TerrainResource::ScatterPoints(_) => ResourceKind::ScatterPoints,
    }
}

/// Resource storage for one terrain pipeline execution.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ResourceSet {
    resources: HashMap<ResourceKey, TerrainResource>,
}

impl ResourceSet {
    /// Creates an empty resource set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of resources stored by key.
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Returns true when no resources are stored.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    /// Returns true when a resource exists at `key`.
    pub fn contains(&self, key: ResourceKey) -> bool {
        self.resources.contains_key(&key)
    }

    /// Returns the resource kind at `key`, if a resource exists.
    pub fn kind(&self, key: ResourceKey) -> Option<ResourceKind> {
        self.get(key).map(resource_kind)
    }

    /// Returns a canonical resource by key.
    pub fn get(&self, key: ResourceKey) -> Option<&TerrainResource> {
        self.resources.get(&key)
    }

    /// Returns a mutable canonical resource by key.
    pub fn get_mut(&mut self, key: ResourceKey) -> Option<&mut TerrainResource> {
        self.resources.get_mut(&key)
    }

    /// Inserts a canonical resource, returning the previous resource at `key`.
    pub fn insert(
        &mut self,
        key: ResourceKey,
        resource: TerrainResource,
    ) -> Option<TerrainResource> {
        self.resources.insert(key, resource)
    }

    /// Removes and returns a canonical resource by key.
    pub fn remove(&mut self, key: ResourceKey) -> Option<TerrainResource> {
        self.resources.remove(&key)
    }

    /// Returns a height field when `key` exists with the matching kind.
    pub fn height_field(&self, key: ResourceKey) -> Option<&HeightField> {
        match self.resources.get(&key) {
            Some(TerrainResource::HeightField(resource)) => Some(resource),
            _ => None,
        }
    }

    /// Returns a mutable height field when `key` exists with the matching kind.
    pub fn height_field_mut(&mut self, key: ResourceKey) -> Option<&mut HeightField> {
        match self.resources.get_mut(&key) {
            Some(TerrainResource::HeightField(resource)) => Some(resource),
            _ => None,
        }
    }

    /// Returns a density field when `key` exists with the matching kind.
    pub fn density_field(&self, key: ResourceKey) -> Option<&DensityField> {
        match self.resources.get(&key) {
            Some(TerrainResource::DensityField(resource)) => Some(resource),
            _ => None,
        }
    }

    /// Returns a mutable density field when `key` exists with the matching kind.
    pub fn density_field_mut(&mut self, key: ResourceKey) -> Option<&mut DensityField> {
        match self.resources.get_mut(&key) {
            Some(TerrainResource::DensityField(resource)) => Some(resource),
            _ => None,
        }
    }

    /// Returns a voxel volume when `key` exists with the matching kind.
    pub fn voxel_volume(&self, key: ResourceKey) -> Option<&VoxelVolume> {
        match self.resources.get(&key) {
            Some(TerrainResource::VoxelVolume(resource)) => Some(resource),
            _ => None,
        }
    }

    /// Returns a mutable voxel volume when `key` exists with the matching kind.
    pub fn voxel_volume_mut(&mut self, key: ResourceKey) -> Option<&mut VoxelVolume> {
        match self.resources.get_mut(&key) {
            Some(TerrainResource::VoxelVolume(resource)) => Some(resource),
            _ => None,
        }
    }

    /// Returns a mesh when `key` exists with the matching kind.
    pub fn mesh(&self, key: ResourceKey) -> Option<&TerrainMesh> {
        match self.resources.get(&key) {
            Some(TerrainResource::Mesh(resource)) => Some(resource),
            _ => None,
        }
    }

    /// Returns a mutable mesh when `key` exists with the matching kind.
    pub fn mesh_mut(&mut self, key: ResourceKey) -> Option<&mut TerrainMesh> {
        match self.resources.get_mut(&key) {
            Some(TerrainResource::Mesh(resource)) => Some(resource),
            _ => None,
        }
    }

    /// Returns scattered placements for a matching resource key.
    pub fn scatter_points(&self, key: ResourceKey) -> Option<&ScatterPoints> {
        match self.resources.get(&key) {
            Some(TerrainResource::ScatterPoints(resource)) => Some(resource),
            _ => None,
        }
    }

    /// Returns mutable scattered placements for a matching resource key.
    pub fn scatter_points_mut(&mut self, key: ResourceKey) -> Option<&mut ScatterPoints> {
        match self.resources.get_mut(&key) {
            Some(TerrainResource::ScatterPoints(resource)) => Some(resource),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use terrakit_core::{
        Extent2, GridTransform2, SamplingDomain, Vector3F32, Vector3F64, VoxelVolume,
    };

    fn height_field(value: f32) -> HeightField {
        HeightField::filled(
            Extent2::try_new(2, 2).unwrap(),
            value,
            GridTransform2::identity_xz(),
            SamplingDomain::Points,
            Vector3F64::Y,
        )
        .unwrap()
    }

    fn mesh() -> TerrainMesh {
        TerrainMesh::new(
            Vector3F64::ZERO,
            vec![
                Vector3F32::new(0.0, 0.0, 0.0),
                Vector3F32::new(1.0, 0.0, 0.0),
                Vector3F32::new(0.0, 0.0, 1.0),
            ],
            vec![0, 1, 2],
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn resource_can_be_inserted_and_retrieved() {
        let mut resources = ResourceSet::new();
        let resource = TerrainResource::HeightField(height_field(1.0));

        assert_eq!(
            resources.insert(ResourceKey::HEIGHT, resource.clone()),
            None
        );
        assert_eq!(resources.get(ResourceKey::HEIGHT), Some(&resource));
        assert!(resources.contains(ResourceKey::HEIGHT));
        assert_eq!(resources.len(), 1);
    }

    #[test]
    fn replacing_resource_returns_previous_value() {
        let mut resources = ResourceSet::new();
        let first = TerrainResource::HeightField(height_field(1.0));
        let second = TerrainResource::HeightField(height_field(2.0));

        assert_eq!(resources.insert(ResourceKey::HEIGHT, first.clone()), None);
        assert_eq!(resources.insert(ResourceKey::HEIGHT, second), Some(first));
    }

    #[test]
    fn typed_accessor_returns_correct_resource() {
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field(1.0)),
        );

        assert_eq!(
            resources
                .height_field(ResourceKey::HEIGHT)
                .map(HeightField::values),
            Some(&[1.0, 1.0, 1.0, 1.0][..])
        );
    }

    #[test]
    fn typed_accessor_returns_none_for_wrong_kind() {
        let mut resources = ResourceSet::new();
        resources.insert(ResourceKey::HEIGHT, TerrainResource::Mesh(mesh()));

        assert_eq!(resources.height_field(ResourceKey::HEIGHT), None);
        assert!(resources.mesh(ResourceKey::HEIGHT).is_some());
    }

    #[test]
    fn removing_resource_works() {
        let mut resources = ResourceSet::new();
        let resource = TerrainResource::HeightField(height_field(1.0));

        resources.insert(ResourceKey::HEIGHT, resource.clone());

        assert_eq!(resources.remove(ResourceKey::HEIGHT), Some(resource));
        assert!(resources.is_empty());
    }

    #[test]
    fn resource_kind_detection_is_correct() {
        assert_eq!(
            resource_kind(&TerrainResource::HeightField(height_field(1.0))),
            ResourceKind::HeightField
        );
        assert_eq!(
            resource_kind(&TerrainResource::DensityField(
                DensityField::from_dimensions(1, 1, 1).unwrap()
            )),
            ResourceKind::DensityField
        );
        assert_eq!(
            resource_kind(&TerrainResource::VoxelVolume(
                VoxelVolume::from_dimensions(1, 1, 1).unwrap()
            )),
            ResourceKind::VoxelVolume
        );
        assert_eq!(
            resource_kind(&TerrainResource::Mesh(mesh())),
            ResourceKind::Mesh
        );
    }
}
