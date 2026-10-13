//! Stage construction handle creation, parameter assignment, and port binding.

use terrakit_core::{Vector2F64, Vector3F64};
use terrakit_pipeline::{
    EnumValueId, ParameterId, ParameterSet, ParameterValue, PortId, ResourceKey, StageBindings,
    StageConstruction, StageId, StageSchemaVersion, StageTypeId,
};

use crate::{
    TK_PARAMETER_KIND_BOOL, TK_PARAMETER_KIND_ENUM, TK_PARAMETER_KIND_F32, TK_PARAMETER_KIND_F64,
    TK_PARAMETER_KIND_I64, TK_PARAMETER_KIND_STRING, TK_PARAMETER_KIND_U64,
    TK_PARAMETER_KIND_VECTOR2_F64, TK_PARAMETER_KIND_VECTOR3_F64, TK_STATUS_INVALID_ARGUMENT,
    TkStatus,
    error::{AbiError, binding_error, definition_error, ffi_guard},
    ffi::{
        destroy_handle, mut_from_ptr, ref_from_ptr, require_bool, require_zero_reserved,
        string_view_to_str,
    },
    types::{TkParameterValue, TkStageConstruction, TkStringView},
};

/// Internal storage for a stage construction handle.
#[derive(Debug)]
pub(crate) struct StageConstructionHandle {
    stage_id: StageId,
    stage_type: StageTypeId,
    schema_version: StageSchemaVersion,
    parameters: ParameterSet,
    bindings: StageBindings,
}

impl StageConstructionHandle {
    pub(crate) fn to_construction(&self) -> StageConstruction {
        StageConstruction::new(self.stage_id, self.stage_type.clone(), self.schema_version)
            .with_parameters(self.parameters.clone())
            .with_bindings(self.bindings.clone())
    }
}

/// Creates an owned stage construction handle.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_construction_create(
    stage_id: u64,
    stage_type_id: TkStringView,
    schema_version: u32,
    out_construction: *mut *mut TkStageConstruction,
) -> TkStatus {
    ffi_guard(|| {
        let out = crate::ffi::handle_slot(out_construction, "out_construction")?;
        *out = std::ptr::null_mut();

        let stage_type_text = string_view_to_str(stage_type_id, "stage_type_id")?;
        let stage_type = StageTypeId::try_new(stage_type_text)
            .map_err(|error| definition_error("invalid stage type identifier", error))?;
        let schema_version = StageSchemaVersion::try_new(schema_version)
            .map_err(|error| definition_error("invalid schema version", error))?;

        let handle = Box::new(StageConstructionHandle {
            stage_id: StageId(stage_id),
            stage_type,
            schema_version,
            parameters: ParameterSet::new(),
            bindings: StageBindings::new(),
        });
        *out = Box::into_raw(handle).cast::<TkStageConstruction>();
        Ok(())
    })
}

/// Destroys a stage construction handle.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_construction_destroy(
    construction: *mut *mut TkStageConstruction,
) -> TkStatus {
    ffi_guard(|| {
        destroy_handle::<StageConstructionHandle, TkStageConstruction>(construction, "construction")
    })
}

/// Inserts or replaces one parameter value on a stage construction.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_construction_set_parameter(
    construction: *mut TkStageConstruction,
    parameter_id: TkStringView,
    value: *const TkParameterValue,
) -> TkStatus {
    ffi_guard(|| {
        let construction = mut_from_ptr::<StageConstructionHandle, TkStageConstruction>(
            construction,
            "construction",
        )?;
        let parameter_text = string_view_to_str(parameter_id, "parameter_id")?;
        let parameter = ParameterId::try_new(parameter_text)
            .map_err(|error| definition_error("invalid parameter identifier", error))?;
        let value = ref_from_ptr::<TkParameterValue, TkParameterValue>(value, "value")?;
        let value = parameter_value_from_abi(value)?;
        construction.parameters.set(parameter, value);
        Ok(())
    })
}

/// Binds one input port to a caller-assigned resource key.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_construction_bind_input(
    construction: *mut TkStageConstruction,
    port_id: TkStringView,
    resource_key: u64,
) -> TkStatus {
    ffi_guard(|| {
        let construction = mut_from_ptr::<StageConstructionHandle, TkStageConstruction>(
            construction,
            "construction",
        )?;
        let port = port_from_view(port_id)?;
        construction
            .bindings
            .bind_input(port, ResourceKey(resource_key))
            .map_err(|error| binding_error("failed to bind input", error))?;
        Ok(())
    })
}

/// Binds one output port to a caller-assigned resource key.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_construction_bind_output(
    construction: *mut TkStageConstruction,
    port_id: TkStringView,
    resource_key: u64,
) -> TkStatus {
    ffi_guard(|| {
        let construction = mut_from_ptr::<StageConstructionHandle, TkStageConstruction>(
            construction,
            "construction",
        )?;
        let port = port_from_view(port_id)?;
        construction
            .bindings
            .bind_output(port, ResourceKey(resource_key))
            .map_err(|error| binding_error("failed to bind output", error))?;
        Ok(())
    })
}

pub(crate) fn parameter_value_from_abi(
    value: &TkParameterValue,
) -> Result<ParameterValue, AbiError> {
    validate_reserved_fields(value)?;

    match value.kind {
        TK_PARAMETER_KIND_BOOL => {
            let value = require_bool(value.bool_value, "value.bool_value")?;
            Ok(ParameterValue::Bool(value))
        }
        TK_PARAMETER_KIND_I64 => Ok(ParameterValue::I64(value.i64_value)),
        TK_PARAMETER_KIND_U64 => Ok(ParameterValue::U64(value.u64_value)),
        TK_PARAMETER_KIND_F32 => {
            if !value.f32_value.is_finite() {
                return Err(AbiError::invalid_argument(
                    "f32 parameter values must be finite",
                ));
            }
            Ok(ParameterValue::F32(value.f32_value))
        }
        TK_PARAMETER_KIND_F64 => {
            if !value.f64_value.is_finite() {
                return Err(AbiError::invalid_argument(
                    "f64 parameter values must be finite",
                ));
            }
            Ok(ParameterValue::F64(value.f64_value))
        }
        TK_PARAMETER_KIND_VECTOR2_F64 => {
            let vector = Vector2F64::from(value.vector2_f64_value);
            if !vector.is_finite() {
                return Err(AbiError::invalid_argument(
                    "vector2_f64 parameter values must be finite",
                ));
            }
            Ok(ParameterValue::Vector2F64(vector))
        }
        TK_PARAMETER_KIND_VECTOR3_F64 => {
            let vector = Vector3F64::from(value.vector3_f64_value);
            if !vector.is_finite() {
                return Err(AbiError::invalid_argument(
                    "vector3_f64 parameter values must be finite",
                ));
            }
            Ok(ParameterValue::Vector3F64(vector))
        }
        TK_PARAMETER_KIND_STRING => {
            let text = string_view_to_str(value.text_value, "value.text_value")?;
            Ok(ParameterValue::String(Box::<str>::from(text)))
        }
        TK_PARAMETER_KIND_ENUM => {
            let text = string_view_to_str(value.text_value, "value.text_value")?;
            let enum_id = EnumValueId::try_new(text)
                .map_err(|error| definition_error("invalid enum value identifier", error))?;
            Ok(ParameterValue::Enum(enum_id))
        }
        other => Err(AbiError::new(
            TK_STATUS_INVALID_ARGUMENT,
            format!("invalid parameter kind discriminant {other}"),
        )),
    }
}

fn validate_reserved_fields(value: &TkParameterValue) -> Result<(), AbiError> {
    require_zero_reserved(&value.reserved_bool, "value.reserved_bool")?;
    if value.reserved_f32 != 0 {
        return Err(AbiError::invalid_argument(
            "reserved field 'value.reserved_f32' must be zero",
        ));
    }
    Ok(())
}

fn port_from_view(port_id: TkStringView) -> Result<PortId, AbiError> {
    let port_text = string_view_to_str(port_id, "port_id")?;
    PortId::try_new(port_text).map_err(|error| definition_error("invalid port identifier", error))
}
