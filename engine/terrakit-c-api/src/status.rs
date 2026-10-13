//! Stable integer status, boolean, and tag values used by the C ABI.

/// ABI status code returned by fallible TerraKit C API functions.
pub type TkStatus = i32;

/// ABI boolean value. Valid values are `TK_FALSE` and `TK_TRUE`.
pub type TkBool = u8;

/// False ABI boolean value.
pub const TK_FALSE: TkBool = 0;
/// True ABI boolean value.
pub const TK_TRUE: TkBool = 1;

/// Successful call.
pub const TK_STATUS_OK: TkStatus = 0;
/// A required pointer argument was null.
pub const TK_STATUS_NULL_ARGUMENT: TkStatus = 1;
/// A value was malformed or structurally invalid.
pub const TK_STATUS_INVALID_ARGUMENT: TkStatus = 2;
/// A string view did not contain UTF-8.
pub const TK_STATUS_INVALID_UTF8: TkStatus = 3;
/// A numeric value or index was outside the accepted range.
pub const TK_STATUS_OUT_OF_RANGE: TkStatus = 4;
/// A requested schema, port, parameter, enum option, or resource was missing.
pub const TK_STATUS_NOT_FOUND: TkStatus = 5;
/// A parameter, resource, request, or view had the wrong kind.
pub const TK_STATUS_TYPE_MISMATCH: TkStatus = 6;
/// A stage construction requested a schema version that is not registered.
pub const TK_STATUS_SCHEMA_VERSION_MISMATCH: TkStatus = 7;
/// A registry, binding, parameter, or pipeline configuration failed.
pub const TK_STATUS_CONFIGURATION_ERROR: TkStatus = 8;
/// Runtime generation failed while executing a pipeline.
pub const TK_STATUS_GENERATION_ERROR: TkStatus = 9;
/// The caller-provided copy buffer was too small.
pub const TK_STATUS_BUFFER_TOO_SMALL: TkStatus = 10;
/// A panic or impossible internal state was caught at the ABI boundary.
pub const TK_STATUS_INTERNAL_ERROR: TkStatus = 255;

/// ABI region-kind tag.
pub type TkRegionKind = i32;

/// Invalid or unknown region kind.
pub const TK_REGION_KIND_INVALID: TkRegionKind = 0;
/// Two-dimensional region kind.
pub const TK_REGION_KIND_REGION_2: TkRegionKind = 1;
/// Three-dimensional region kind.
pub const TK_REGION_KIND_REGION_3: TkRegionKind = 2;

/// ABI resource-kind tag.
pub type TkResourceKind = i32;

/// Invalid or unknown resource kind.
pub const TK_RESOURCE_KIND_INVALID: TkResourceKind = 0;
/// Height-field resource kind.
pub const TK_RESOURCE_KIND_HEIGHT_FIELD: TkResourceKind = 1;
/// Density-field resource kind.
pub const TK_RESOURCE_KIND_DENSITY_FIELD: TkResourceKind = 2;
/// Voxel-volume resource kind.
pub const TK_RESOURCE_KIND_VOXEL_VOLUME: TkResourceKind = 3;
/// Terrain-mesh resource kind.
pub const TK_RESOURCE_KIND_MESH: TkResourceKind = 4;
/// Scatter-point resource kind.
pub const TK_RESOURCE_KIND_SCATTER_POINTS: TkResourceKind = 5;

/// ABI sampling-domain tag.
pub type TkSamplingDomain = i32;

/// Invalid or unknown sampling domain.
pub const TK_SAMPLING_DOMAIN_INVALID: TkSamplingDomain = 0;
/// Point-sampled domain.
pub const TK_SAMPLING_DOMAIN_POINTS: TkSamplingDomain = 1;
/// Cell-sampled domain.
pub const TK_SAMPLING_DOMAIN_CELLS: TkSamplingDomain = 2;

/// ABI parameter-kind tag.
pub type TkParameterKind = i32;

/// Invalid or unknown parameter kind.
pub const TK_PARAMETER_KIND_INVALID: TkParameterKind = 0;
/// Boolean parameter kind.
pub const TK_PARAMETER_KIND_BOOL: TkParameterKind = 1;
/// Signed 64-bit integer parameter kind.
pub const TK_PARAMETER_KIND_I64: TkParameterKind = 2;
/// Unsigned 64-bit integer parameter kind.
pub const TK_PARAMETER_KIND_U64: TkParameterKind = 3;
/// 32-bit float parameter kind.
pub const TK_PARAMETER_KIND_F32: TkParameterKind = 4;
/// 64-bit float parameter kind.
pub const TK_PARAMETER_KIND_F64: TkParameterKind = 5;
/// Two-component `f64` vector parameter kind.
pub const TK_PARAMETER_KIND_VECTOR2_F64: TkParameterKind = 6;
/// Three-component `f64` vector parameter kind.
pub const TK_PARAMETER_KIND_VECTOR3_F64: TkParameterKind = 7;
/// UTF-8 string parameter kind.
pub const TK_PARAMETER_KIND_STRING: TkParameterKind = 8;
/// Stable enum-option identifier parameter kind.
pub const TK_PARAMETER_KIND_ENUM: TkParameterKind = 9;

pub(crate) fn bool_to_tk(value: bool) -> TkBool {
    if value { TK_TRUE } else { TK_FALSE }
}
