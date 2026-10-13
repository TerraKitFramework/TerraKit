//! C-layout ABI value, metadata, request, and view structures.

use std::{ffi::c_char, mem};

use terrakit_core::{
    GridTransform2, GridTransform3, SamplingDomain, Vector2F32, Vector2F64, Vector3F32, Vector3F64,
};
use terrakit_pipeline::{ParameterKind, ResourceKind};
use terrakit_runtime::GenerationRegionKind;

use crate::{
    TK_PARAMETER_KIND_BOOL, TK_PARAMETER_KIND_ENUM, TK_PARAMETER_KIND_F32, TK_PARAMETER_KIND_F64,
    TK_PARAMETER_KIND_I64, TK_PARAMETER_KIND_STRING, TK_PARAMETER_KIND_U64,
    TK_PARAMETER_KIND_VECTOR2_F64, TK_PARAMETER_KIND_VECTOR3_F64, TK_REGION_KIND_REGION_2,
    TK_REGION_KIND_REGION_3, TK_RESOURCE_KIND_DENSITY_FIELD, TK_RESOURCE_KIND_HEIGHT_FIELD,
    TK_RESOURCE_KIND_MESH, TK_RESOURCE_KIND_SCATTER_POINTS, TK_RESOURCE_KIND_VOXEL_VOLUME,
    TK_SAMPLING_DOMAIN_CELLS, TK_SAMPLING_DOMAIN_POINTS, TkBool, TkParameterKind, TkRegionKind,
    TkResourceKind, TkSamplingDomain,
};

/// Opaque stage registry handle.
pub struct TkStageRegistry {
    _private: [u8; 0],
}

/// Opaque stage construction handle.
pub struct TkStageConstruction {
    _private: [u8; 0],
}

/// Opaque pipeline assembler handle.
pub struct TkPipelineAssembler {
    _private: [u8; 0],
}

/// Opaque completed pipeline handle.
pub struct TkPipeline {
    _private: [u8; 0],
}

/// Opaque synchronous runtime handle.
pub struct TkRuntime {
    _private: [u8; 0],
}

/// Opaque generation result handle.
pub struct TkGenerationResult {
    _private: [u8; 0],
}

/// C-layout semantic version.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TkVersion {
    /// Major version component.
    pub major: u32,
    /// Minor version component.
    pub minor: u32,
    /// Patch version component.
    pub patch: u32,
}

/// Borrowed UTF-8 string view.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TkStringView {
    /// Pointer to UTF-8 bytes, not necessarily NUL-terminated.
    pub data: *const c_char,
    /// Number of bytes available at `data`.
    pub length: usize,
}

impl Default for TkStringView {
    fn default() -> Self {
        Self {
            data: std::ptr::null(),
            length: 0,
        }
    }
}

impl TkStringView {
    pub(crate) fn from_str(value: &str) -> Self {
        if value.is_empty() {
            Self::default()
        } else {
            Self {
                data: value.as_ptr().cast::<c_char>(),
                length: value.len(),
            }
        }
    }
}

/// C-layout two-component 32-bit vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkVec2F32 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
}

/// C-layout three-component 32-bit vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkVec3F32 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
    /// Z component.
    pub z: f32,
}

/// C-layout two-component 64-bit vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkVec2F64 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
}

/// C-layout three-component 64-bit vector.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkVec3F64 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
    /// Z component.
    pub z: f64,
}

impl From<Vector2F32> for TkVec2F32 {
    fn from(value: Vector2F32) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<TkVec2F32> for Vector2F32 {
    fn from(value: TkVec2F32) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<Vector3F32> for TkVec3F32 {
    fn from(value: Vector3F32) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<TkVec3F32> for Vector3F32 {
    fn from(value: TkVec3F32) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<Vector2F64> for TkVec2F64 {
    fn from(value: Vector2F64) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

impl From<TkVec2F64> for Vector2F64 {
    fn from(value: TkVec2F64) -> Self {
        Self::new(value.x, value.y)
    }
}

impl From<Vector3F64> for TkVec3F64 {
    fn from(value: Vector3F64) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<TkVec3F64> for Vector3F64 {
    fn from(value: TkVec3F64) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

/// C-layout 2D grid-to-world transform.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkGridTransform2 {
    /// World-space grid origin.
    pub origin: TkVec3F64,
    /// World-space axis for increasing grid X.
    pub axis_x: TkVec3F64,
    /// World-space axis for increasing grid Y.
    pub axis_y: TkVec3F64,
}

impl From<GridTransform2> for TkGridTransform2 {
    fn from(value: GridTransform2) -> Self {
        Self {
            origin: value.origin().into(),
            axis_x: value.axis_x().into(),
            axis_y: value.axis_y().into(),
        }
    }
}

/// C-layout 3D grid-to-world transform.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkGridTransform3 {
    /// World-space grid origin.
    pub origin: TkVec3F64,
    /// World-space axis for increasing grid X.
    pub axis_x: TkVec3F64,
    /// World-space axis for increasing grid Y.
    pub axis_y: TkVec3F64,
    /// World-space axis for increasing grid Z.
    pub axis_z: TkVec3F64,
}

impl From<GridTransform3> for TkGridTransform3 {
    fn from(value: GridTransform3) -> Self {
        Self {
            origin: value.origin().into(),
            axis_x: value.axis_x().into(),
            axis_y: value.axis_y().into(),
            axis_z: value.axis_z().into(),
        }
    }
}

/// Discoverable metadata for one stage schema.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TkStageSchemaInfo {
    /// Stable stage type identifier.
    pub type_id: TkStringView,
    /// Stage schema version.
    pub schema_version: u32,
    /// User-facing stage name.
    pub display_name: TkStringView,
    /// User-facing category label.
    pub category: TkStringView,
    /// User-facing description.
    pub description: TkStringView,
    /// Number of input ports.
    pub input_count: usize,
    /// Number of output ports.
    pub output_count: usize,
    /// Number of editable parameters.
    pub parameter_count: usize,
}

/// Discoverable metadata for one input port.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TkInputPortInfo {
    /// Stable local port identifier.
    pub id: TkStringView,
    /// User-facing port name.
    pub display_name: TkStringView,
    /// User-facing description.
    pub description: TkStringView,
    /// Accepted resource kind.
    pub resource_kind: TkResourceKind,
    /// Nonzero when the input may be unbound.
    pub optional: TkBool,
}

/// Discoverable metadata for one output port.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TkOutputPortInfo {
    /// Stable local port identifier.
    pub id: TkStringView,
    /// User-facing port name.
    pub display_name: TkStringView,
    /// User-facing description.
    pub description: TkStringView,
    /// Produced resource kind.
    pub resource_kind: TkResourceKind,
}

/// Discoverable metadata for one enum option.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TkEnumOptionInfo {
    /// Stable enum-option identifier.
    pub id: TkStringView,
    /// User-facing option name.
    pub display_name: TkStringView,
    /// User-facing description.
    pub description: TkStringView,
}

/// Tagged ABI parameter value.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TkParameterValue {
    /// Active value kind.
    pub kind: TkParameterKind,
    /// Boolean value for `TK_PARAMETER_KIND_BOOL`.
    pub bool_value: TkBool,
    /// Reserved boolean padding bytes. Must be zero.
    pub reserved_bool: [u8; 3],
    /// Signed integer value for `TK_PARAMETER_KIND_I64`.
    pub i64_value: i64,
    /// Unsigned integer value for `TK_PARAMETER_KIND_U64`.
    pub u64_value: u64,
    /// 32-bit float value for `TK_PARAMETER_KIND_F32`.
    pub f32_value: f32,
    /// Reserved float padding. Must be zero.
    pub reserved_f32: u32,
    /// 64-bit float value for `TK_PARAMETER_KIND_F64`.
    pub f64_value: f64,
    /// Two-component vector value for `TK_PARAMETER_KIND_VECTOR2_F64`.
    pub vector2_f64_value: TkVec2F64,
    /// Three-component vector value for `TK_PARAMETER_KIND_VECTOR3_F64`.
    pub vector3_f64_value: TkVec3F64,
    /// UTF-8 text for string and enum values.
    pub text_value: TkStringView,
}

/// Discoverable parameter metadata, defaults, bounds, and enum option count.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TkParameterInfo {
    /// Stable parameter identifier.
    pub id: TkStringView,
    /// User-facing parameter name.
    pub display_name: TkStringView,
    /// User-facing description.
    pub description: TkStringView,
    /// Accepted parameter kind.
    pub kind: TkParameterKind,
    /// Nonzero when a default value is present.
    pub has_default: TkBool,
    /// Default value when `has_default` is `TK_TRUE`.
    pub default_value: TkParameterValue,
    /// Nonzero when a numeric minimum exists.
    pub has_minimum: TkBool,
    /// Inclusive numeric minimum when present.
    pub minimum_value: TkParameterValue,
    /// Nonzero when a numeric maximum exists.
    pub has_maximum: TkBool,
    /// Inclusive numeric maximum when present.
    pub maximum_value: TkParameterValue,
    /// Number of enum options for enum parameters.
    pub enum_option_count: usize,
}

/// 2D region layout for runtime creation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkRegionLayout2 {
    /// Highest-detail cell width.
    pub cell_width: u32,
    /// Highest-detail cell height.
    pub cell_height: u32,
    /// Highest-detail cell spacing.
    pub base_spacing: TkVec2F64,
}

/// 3D region layout for runtime creation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkRegionLayout3 {
    /// Highest-detail cell width.
    pub cell_width: u32,
    /// Highest-detail cell height.
    pub cell_height: u32,
    /// Highest-detail cell depth.
    pub cell_depth: u32,
    /// Reserved field. Must be zero.
    pub reserved: u32,
    /// Highest-detail cell spacing.
    pub base_spacing: TkVec3F64,
}

/// Per-call 2D generation request.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TkGenerationRequest2 {
    /// Deterministic generation seed.
    pub seed: u64,
    /// Region coordinate X.
    pub region_x: i64,
    /// Region coordinate Y.
    pub region_y: i64,
    /// Opaque generation-detail tier.
    pub lod_level: u16,
    /// Reserved bytes. Must be zero.
    pub reserved: [u8; 6],
}

/// Per-call 3D generation request.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TkGenerationRequest3 {
    /// Deterministic generation seed.
    pub seed: u64,
    /// Region coordinate X.
    pub region_x: i64,
    /// Region coordinate Y.
    pub region_y: i64,
    /// Region coordinate Z.
    pub region_z: i64,
    /// Opaque generation-detail tier.
    pub lod_level: u16,
    /// Reserved bytes. Must be zero.
    pub reserved: [u8; 6],
}

/// Resolved 2D region information returned by a generation result.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkRegion2Info {
    /// Region coordinate X.
    pub region_x: i64,
    /// Region coordinate Y.
    pub region_y: i64,
    /// Opaque generation-detail tier.
    pub lod_level: u16,
    /// Reserved bytes. Always zero on output.
    pub reserved: [u8; 6],
    /// Resolved cell-sample width.
    pub cell_width: u32,
    /// Resolved cell-sample height.
    pub cell_height: u32,
    /// Resolved point-sample width.
    pub point_width: u32,
    /// Resolved point-sample height.
    pub point_height: u32,
    /// Resolved effective point spacing.
    pub effective_spacing: TkVec2F64,
    /// Resolved grid-to-world transform.
    pub transform: TkGridTransform2,
}

/// Resolved 3D region information returned by a generation result.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkRegion3Info {
    /// Region coordinate X.
    pub region_x: i64,
    /// Region coordinate Y.
    pub region_y: i64,
    /// Region coordinate Z.
    pub region_z: i64,
    /// Opaque generation-detail tier.
    pub lod_level: u16,
    /// Reserved bytes. Always zero on output.
    pub reserved: [u8; 6],
    /// Resolved cell-sample width.
    pub cell_width: u32,
    /// Resolved cell-sample height.
    pub cell_height: u32,
    /// Resolved cell-sample depth.
    pub cell_depth: u32,
    /// Reserved output extent field. Always zero.
    pub reserved_extent: u32,
    /// Resolved point-sample width.
    pub point_width: u32,
    /// Resolved point-sample height.
    pub point_height: u32,
    /// Resolved point-sample depth.
    pub point_depth: u32,
    /// Reserved output point extent field. Always zero.
    pub reserved_point_extent: u32,
    /// Resolved effective point spacing.
    pub effective_spacing: TkVec3F64,
    /// Resolved grid-to-world transform.
    pub transform: TkGridTransform3,
}

/// Borrowed immutable height-field view.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TkHeightFieldView {
    /// Height-field sample width.
    pub width: u32,
    /// Height-field sample height.
    pub height: u32,
    /// Sampling-domain tag.
    pub sampling: TkSamplingDomain,
    /// Reserved output field. Always zero.
    pub reserved: u32,
    /// Base grid transform.
    pub transform: TkGridTransform2,
    /// Height displacement axis.
    pub height_axis: TkVec3F64,
    /// Borrowed contiguous sample pointer.
    pub values: *const f32,
    /// Number of samples at `values`.
    pub value_count: usize,
}

impl Default for TkHeightFieldView {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            sampling: 0,
            reserved: 0,
            transform: TkGridTransform2::default(),
            height_axis: TkVec3F64::default(),
            values: std::ptr::null(),
            value_count: 0,
        }
    }
}

/// One engine-independent object placement point.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TkScatterPoint {
    /// Position relative to TkScatterPointsView.origin (x,y,z).
    pub position: [f32; 3],
    /// Quaternion rotation (x,y,z,w)
    pub rotation: [f32; 4],
    /// X,Y,Z scales
    pub scale: [f32; 3],
    /// Identifier of the object prototype to instantiate.
    pub prototype_id: u32,
}

/// Borrowed immutable scatter-point view.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TkScatterPointsView {
    /// Precise world-space origin to add to each local placement.
    pub origin: TkVec3F64,
    /// Borrowed contiguous scatter-point pointer.
    pub points: *const TkScatterPoint,
    /// Number of scatter points.
    pub point_count: usize,
}

impl Default for TkScatterPointsView {
    fn default() -> Self {
        Self {
            origin: TkVec3F64::default(),
            points: std::ptr::null(),
            point_count: 0,
        }
    }
}

/// Borrowed immutable density-field view.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TkDensityFieldView {
    /// Density-field sample width.
    pub width: u32,
    /// Density-field sample height.
    pub height: u32,
    /// Density-field sample depth.
    pub depth: u32,
    /// Reserved output extent field. Always zero.
    pub reserved_extent: u32,
    /// Sampling-domain tag.
    pub sampling: TkSamplingDomain,
    /// Reserved output sampling field. Always zero.
    pub reserved_sampling: u32,
    /// Grid transform.
    pub transform: TkGridTransform3,
    /// Borrowed contiguous sample pointer.
    pub values: *const f32,
    /// Number of samples at `values`.
    pub value_count: usize,
}

impl Default for TkDensityFieldView {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            depth: 0,
            reserved_extent: 0,
            sampling: 0,
            reserved_sampling: 0,
            transform: TkGridTransform3::default(),
            values: std::ptr::null(),
            value_count: 0,
        }
    }
}

/// Borrowed immutable voxel-volume view.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TkVoxelVolumeView {
    /// Voxel-volume sample width.
    pub width: u32,
    /// Voxel-volume sample height.
    pub height: u32,
    /// Voxel-volume sample depth.
    pub depth: u32,
    /// Reserved output extent field. Always zero.
    pub reserved_extent: u32,
    /// Sampling-domain tag.
    pub sampling: TkSamplingDomain,
    /// Reserved output sampling field. Always zero.
    pub reserved_sampling: u32,
    /// Grid transform.
    pub transform: TkGridTransform3,
    /// Borrowed contiguous raw voxel ID pointer.
    pub values: *const u32,
    /// Number of voxel IDs at `values`.
    pub value_count: usize,
}

impl Default for TkVoxelVolumeView {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            depth: 0,
            reserved_extent: 0,
            sampling: 0,
            reserved_sampling: 0,
            transform: TkGridTransform3::default(),
            values: std::ptr::null(),
            value_count: 0,
        }
    }
}

/// Borrowed immutable terrain-mesh view.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TkTerrainMeshView {
    /// High-precision world origin for local vertex buffers.
    pub origin: TkVec3F64,
    /// Borrowed local-position pointer.
    pub positions: *const TkVec3F32,
    /// Number of positions.
    pub position_count: usize,
    /// Borrowed triangle-index pointer.
    pub indices: *const u32,
    /// Number of triangle indices.
    pub index_count: usize,
    /// Optional borrowed normal pointer.
    pub normals: *const TkVec3F32,
    /// Number of normals, or zero when absent.
    pub normal_count: usize,
    /// Optional borrowed texture-coordinate pointer.
    pub texcoords: *const TkVec2F32,
    /// Number of texture coordinates, or zero when absent.
    pub texcoord_count: usize,
}

impl Default for TkTerrainMeshView {
    fn default() -> Self {
        Self {
            origin: TkVec3F64::default(),
            positions: std::ptr::null(),
            position_count: 0,
            indices: std::ptr::null(),
            index_count: 0,
            normals: std::ptr::null(),
            normal_count: 0,
            texcoords: std::ptr::null(),
            texcoord_count: 0,
        }
    }
}

pub(crate) fn resource_kind_to_tk(kind: ResourceKind) -> TkResourceKind {
    match kind {
        ResourceKind::HeightField => TK_RESOURCE_KIND_HEIGHT_FIELD,
        ResourceKind::DensityField => TK_RESOURCE_KIND_DENSITY_FIELD,
        ResourceKind::VoxelVolume => TK_RESOURCE_KIND_VOXEL_VOLUME,
        ResourceKind::Mesh => TK_RESOURCE_KIND_MESH,
        ResourceKind::ScatterPoints => TK_RESOURCE_KIND_SCATTER_POINTS,
    }
}

pub(crate) fn sampling_domain_to_tk(domain: SamplingDomain) -> TkSamplingDomain {
    match domain {
        SamplingDomain::Points => TK_SAMPLING_DOMAIN_POINTS,
        SamplingDomain::Cells => TK_SAMPLING_DOMAIN_CELLS,
    }
}

pub(crate) fn parameter_kind_to_tk(kind: ParameterKind) -> TkParameterKind {
    match kind {
        ParameterKind::Bool => TK_PARAMETER_KIND_BOOL,
        ParameterKind::I64 => TK_PARAMETER_KIND_I64,
        ParameterKind::U64 => TK_PARAMETER_KIND_U64,
        ParameterKind::F32 => TK_PARAMETER_KIND_F32,
        ParameterKind::F64 => TK_PARAMETER_KIND_F64,
        ParameterKind::Vector2F64 => TK_PARAMETER_KIND_VECTOR2_F64,
        ParameterKind::Vector3F64 => TK_PARAMETER_KIND_VECTOR3_F64,
        ParameterKind::String => TK_PARAMETER_KIND_STRING,
        ParameterKind::Enum => TK_PARAMETER_KIND_ENUM,
    }
}

pub(crate) fn region_kind_to_tk(kind: GenerationRegionKind) -> TkRegionKind {
    match kind {
        GenerationRegionKind::Region2 => TK_REGION_KIND_REGION_2,
        GenerationRegionKind::Region3 => TK_REGION_KIND_REGION_3,
    }
}

const _: () = {
    assert!(mem::size_of::<TkVec2F32>() == mem::size_of::<Vector2F32>());
    assert!(mem::align_of::<TkVec2F32>() == mem::align_of::<Vector2F32>());
    assert!(mem::offset_of!(TkVec2F32, x) == mem::offset_of!(Vector2F32, x));
    assert!(mem::offset_of!(TkVec2F32, y) == mem::offset_of!(Vector2F32, y));

    assert!(mem::size_of::<TkVec3F32>() == mem::size_of::<Vector3F32>());
    assert!(mem::align_of::<TkVec3F32>() == mem::align_of::<Vector3F32>());
    assert!(mem::offset_of!(TkVec3F32, x) == mem::offset_of!(Vector3F32, x));
    assert!(mem::offset_of!(TkVec3F32, y) == mem::offset_of!(Vector3F32, y));
    assert!(mem::offset_of!(TkVec3F32, z) == mem::offset_of!(Vector3F32, z));

    assert!(mem::size_of::<TkVec2F64>() == mem::size_of::<Vector2F64>());
    assert!(mem::align_of::<TkVec2F64>() == mem::align_of::<Vector2F64>());
    assert!(mem::offset_of!(TkVec2F64, x) == mem::offset_of!(Vector2F64, x));
    assert!(mem::offset_of!(TkVec2F64, y) == mem::offset_of!(Vector2F64, y));

    assert!(mem::size_of::<TkVec3F64>() == mem::size_of::<Vector3F64>());
    assert!(mem::align_of::<TkVec3F64>() == mem::align_of::<Vector3F64>());
    assert!(mem::offset_of!(TkVec3F64, x) == mem::offset_of!(Vector3F64, x));
    assert!(mem::offset_of!(TkVec3F64, y) == mem::offset_of!(Vector3F64, y));
    assert!(mem::offset_of!(TkVec3F64, z) == mem::offset_of!(Vector3F64, z));

    assert!(mem::size_of::<u32>() == mem::size_of::<terrakit_core::MeshIndex>());
    assert!(mem::align_of::<u32>() == mem::align_of::<terrakit_core::MeshIndex>());
    assert!(mem::size_of::<u32>() == mem::size_of::<terrakit_core::VoxelId>());
    assert!(mem::align_of::<u32>() == mem::align_of::<terrakit_core::VoxelId>());
};
