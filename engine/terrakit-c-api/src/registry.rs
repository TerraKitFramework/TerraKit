//! Built-in registry creation and immutable stage-schema discovery.

use std::sync::Arc;

use terrakit_builtins::builtin_stage_registry;
use terrakit_pipeline::{
    EnumOption, ParameterDefinition, ParameterType, ParameterValue, StageRegistry, StageSchema,
    StageTypeId,
};

use crate::{
    TK_FALSE, TK_PARAMETER_KIND_BOOL, TK_PARAMETER_KIND_ENUM, TK_PARAMETER_KIND_F32,
    TK_PARAMETER_KIND_F64, TK_PARAMETER_KIND_I64, TK_PARAMETER_KIND_STRING, TK_PARAMETER_KIND_U64,
    TK_PARAMETER_KIND_VECTOR2_F64, TK_PARAMETER_KIND_VECTOR3_F64, TK_TRUE, TkStatus,
    error::{AbiError, definition_error, ffi_guard},
    ffi::{destroy_handle, out_ref, ref_from_ptr, string_view_to_str},
    status::bool_to_tk,
    types::{
        TkEnumOptionInfo, TkInputPortInfo, TkOutputPortInfo, TkParameterInfo, TkParameterValue,
        TkStageRegistry, TkStageSchemaInfo, TkStringView, TkVec2F64, TkVec3F64,
        parameter_kind_to_tk, resource_kind_to_tk,
    },
};

/// Internal storage for a stage registry handle.
pub(crate) struct StageRegistryHandle {
    pub(crate) registry: Arc<StageRegistry>,
}

/// Creates an immutable registry populated with TerraKit built-in stage definitions.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_registry_create_builtin(
    out_registry: *mut *mut TkStageRegistry,
) -> TkStatus {
    ffi_guard(|| {
        let out = crate::ffi::handle_slot(out_registry, "out_registry")?;
        *out = std::ptr::null_mut();

        let registry = builtin_stage_registry().map_err(|error| {
            AbiError::configuration(format!("failed to create registry: {error}"))
        })?;
        let handle = Box::new(StageRegistryHandle {
            registry: Arc::new(registry),
        });
        *out = Box::into_raw(handle).cast::<TkStageRegistry>();
        Ok(())
    })
}

/// Destroys a stage registry handle.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_registry_destroy(registry: *mut *mut TkStageRegistry) -> TkStatus {
    ffi_guard(|| destroy_handle::<StageRegistryHandle, TkStageRegistry>(registry, "registry"))
}

/// Returns the number of schemas in deterministic registry order.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_registry_get_schema_count(
    registry: *const TkStageRegistry,
    out_count: *mut usize,
) -> TkStatus {
    ffi_guard(|| {
        let registry = registry_handle(registry)?;
        let out = out_ref(out_count, "out_count")?;
        *out = registry.registry.len();
        Ok(())
    })
}

/// Finds a schema index by stage type identifier.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_registry_find_schema(
    registry: *const TkStageRegistry,
    stage_type_id: TkStringView,
    out_schema_index: *mut usize,
) -> TkStatus {
    ffi_guard(|| {
        let registry = registry_handle(registry)?;
        let out = out_ref(out_schema_index, "out_schema_index")?;
        let stage_type_text = string_view_to_str(stage_type_id, "stage_type_id")?;
        let stage_type = StageTypeId::try_new(stage_type_text)
            .map_err(|error| definition_error("invalid stage type identifier", error))?;

        let index = registry
            .registry
            .schemas()
            .position(|schema| schema.type_id() == &stage_type)
            .ok_or_else(|| {
                AbiError::not_found(format!("stage type '{stage_type}' was not found"))
            })?;

        *out = index;
        Ok(())
    })
}

/// Returns one schema by deterministic registry index.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_registry_get_schema(
    registry: *const TkStageRegistry,
    schema_index: usize,
    out_schema: *mut TkStageSchemaInfo,
) -> TkStatus {
    ffi_guard(|| {
        let registry = registry_handle(registry)?;
        let schema = schema_at(&registry.registry, schema_index)?;
        let out = out_ref(out_schema, "out_schema")?;
        *out = TkStageSchemaInfo::default();
        *out = schema_info(schema);
        Ok(())
    })
}

/// Returns one input port from a schema.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_registry_get_input_port(
    registry: *const TkStageRegistry,
    schema_index: usize,
    input_index: usize,
    out_port: *mut TkInputPortInfo,
) -> TkStatus {
    ffi_guard(|| {
        let registry = registry_handle(registry)?;
        let schema = schema_at(&registry.registry, schema_index)?;
        let port = schema.inputs().get(input_index).ok_or_else(|| {
            AbiError::out_of_range(format!(
                "input port index {input_index} is out of range for schema {schema_index}"
            ))
        })?;
        let out = out_ref(out_port, "out_port")?;
        *out = TkInputPortInfo::default();
        *out = TkInputPortInfo {
            id: TkStringView::from_str(port.id().as_str()),
            display_name: TkStringView::from_str(port.display_name()),
            description: TkStringView::from_str(port.description()),
            resource_kind: resource_kind_to_tk(port.resource_kind()),
            optional: bool_to_tk(port.optional()),
        };
        Ok(())
    })
}

/// Returns one output port from a schema.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_registry_get_output_port(
    registry: *const TkStageRegistry,
    schema_index: usize,
    output_index: usize,
    out_port: *mut TkOutputPortInfo,
) -> TkStatus {
    ffi_guard(|| {
        let registry = registry_handle(registry)?;
        let schema = schema_at(&registry.registry, schema_index)?;
        let port = schema.outputs().get(output_index).ok_or_else(|| {
            AbiError::out_of_range(format!(
                "output port index {output_index} is out of range for schema {schema_index}"
            ))
        })?;
        let out = out_ref(out_port, "out_port")?;
        *out = TkOutputPortInfo::default();
        *out = TkOutputPortInfo {
            id: TkStringView::from_str(port.id().as_str()),
            display_name: TkStringView::from_str(port.display_name()),
            description: TkStringView::from_str(port.description()),
            resource_kind: resource_kind_to_tk(port.resource_kind()),
        };
        Ok(())
    })
}

/// Returns one parameter definition from a schema.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_registry_get_parameter(
    registry: *const TkStageRegistry,
    schema_index: usize,
    parameter_index: usize,
    out_parameter: *mut TkParameterInfo,
) -> TkStatus {
    ffi_guard(|| {
        let registry = registry_handle(registry)?;
        let schema = schema_at(&registry.registry, schema_index)?;
        let parameter = schema.parameters().get(parameter_index).ok_or_else(|| {
            AbiError::out_of_range(format!(
                "parameter index {parameter_index} is out of range for schema {schema_index}"
            ))
        })?;
        let out = out_ref(out_parameter, "out_parameter")?;
        *out = TkParameterInfo::default();
        *out = parameter_info(parameter);
        Ok(())
    })
}

/// Returns one enum option from an enum parameter.
#[unsafe(no_mangle)]
pub extern "C" fn tk_stage_registry_get_enum_option(
    registry: *const TkStageRegistry,
    schema_index: usize,
    parameter_index: usize,
    option_index: usize,
    out_option: *mut TkEnumOptionInfo,
) -> TkStatus {
    ffi_guard(|| {
        let registry = registry_handle(registry)?;
        let schema = schema_at(&registry.registry, schema_index)?;
        let parameter = schema.parameters().get(parameter_index).ok_or_else(|| {
            AbiError::out_of_range(format!(
                "parameter index {parameter_index} is out of range for schema {schema_index}"
            ))
        })?;
        let options = match parameter.value_type() {
            ParameterType::Enum { options } => options,
            _ => {
                return Err(AbiError::type_mismatch(format!(
                    "parameter '{}' is not an enum parameter",
                    parameter.id()
                )));
            }
        };
        let option = options.get(option_index).ok_or_else(|| {
            AbiError::out_of_range(format!(
                "enum option index {option_index} is out of range for parameter '{}'",
                parameter.id()
            ))
        })?;
        let out = out_ref(out_option, "out_option")?;
        *out = TkEnumOptionInfo::default();
        *out = enum_option_info(option);
        Ok(())
    })
}

pub(crate) fn registry_handle<'a>(
    registry: *const TkStageRegistry,
) -> Result<&'a StageRegistryHandle, AbiError> {
    ref_from_ptr::<StageRegistryHandle, TkStageRegistry>(registry, "registry")
}

pub(crate) fn schema_at(
    registry: &StageRegistry,
    schema_index: usize,
) -> Result<&StageSchema, AbiError> {
    registry.schemas().nth(schema_index).ok_or_else(|| {
        AbiError::out_of_range(format!("schema index {schema_index} is out of range"))
    })
}

pub(crate) fn parameter_value_view(value: &ParameterValue) -> TkParameterValue {
    match value {
        ParameterValue::Bool(value) => TkParameterValue {
            kind: TK_PARAMETER_KIND_BOOL,
            bool_value: bool_to_tk(*value),
            ..TkParameterValue::default()
        },
        ParameterValue::I64(value) => TkParameterValue {
            kind: TK_PARAMETER_KIND_I64,
            i64_value: *value,
            ..TkParameterValue::default()
        },
        ParameterValue::U64(value) => TkParameterValue {
            kind: TK_PARAMETER_KIND_U64,
            u64_value: *value,
            ..TkParameterValue::default()
        },
        ParameterValue::F32(value) => TkParameterValue {
            kind: TK_PARAMETER_KIND_F32,
            f32_value: *value,
            ..TkParameterValue::default()
        },
        ParameterValue::F64(value) => TkParameterValue {
            kind: TK_PARAMETER_KIND_F64,
            f64_value: *value,
            ..TkParameterValue::default()
        },
        ParameterValue::Vector2F64(value) => TkParameterValue {
            kind: TK_PARAMETER_KIND_VECTOR2_F64,
            vector2_f64_value: TkVec2F64::from(*value),
            ..TkParameterValue::default()
        },
        ParameterValue::Vector3F64(value) => TkParameterValue {
            kind: TK_PARAMETER_KIND_VECTOR3_F64,
            vector3_f64_value: TkVec3F64::from(*value),
            ..TkParameterValue::default()
        },
        ParameterValue::String(value) => TkParameterValue {
            kind: TK_PARAMETER_KIND_STRING,
            text_value: TkStringView::from_str(value),
            ..TkParameterValue::default()
        },
        ParameterValue::Enum(value) => TkParameterValue {
            kind: TK_PARAMETER_KIND_ENUM,
            text_value: TkStringView::from_str(value.as_str()),
            ..TkParameterValue::default()
        },
    }
}

fn schema_info(schema: &StageSchema) -> TkStageSchemaInfo {
    TkStageSchemaInfo {
        type_id: TkStringView::from_str(schema.type_id().as_str()),
        schema_version: schema.version().value(),
        display_name: TkStringView::from_str(schema.display_name()),
        category: TkStringView::from_str(schema.category()),
        description: TkStringView::from_str(schema.description()),
        input_count: schema.inputs().len(),
        output_count: schema.outputs().len(),
        parameter_count: schema.parameters().len(),
    }
}

fn parameter_info(parameter: &ParameterDefinition) -> TkParameterInfo {
    let mut info = TkParameterInfo {
        id: TkStringView::from_str(parameter.id().as_str()),
        display_name: TkStringView::from_str(parameter.display_name()),
        description: TkStringView::from_str(parameter.description()),
        kind: parameter_kind_to_tk(parameter.value_type().kind()),
        has_default: TK_FALSE,
        default_value: TkParameterValue::default(),
        has_minimum: TK_FALSE,
        minimum_value: TkParameterValue::default(),
        has_maximum: TK_FALSE,
        maximum_value: TkParameterValue::default(),
        enum_option_count: 0,
    };

    if let Some(default) = parameter.default() {
        info.has_default = TK_TRUE;
        info.default_value = parameter_value_view(default);
    }

    apply_parameter_constraints(&mut info, parameter.value_type());

    info
}

fn apply_parameter_constraints(info: &mut TkParameterInfo, value_type: &ParameterType) {
    match value_type {
        ParameterType::I64 { minimum, maximum } => {
            apply_minimum(
                info,
                minimum.map(|value| TkParameterValue {
                    kind: TK_PARAMETER_KIND_I64,
                    i64_value: value,
                    ..TkParameterValue::default()
                }),
            );
            apply_maximum(
                info,
                maximum.map(|value| TkParameterValue {
                    kind: TK_PARAMETER_KIND_I64,
                    i64_value: value,
                    ..TkParameterValue::default()
                }),
            );
        }
        ParameterType::U64 { minimum, maximum } => {
            apply_minimum(
                info,
                minimum.map(|value| TkParameterValue {
                    kind: TK_PARAMETER_KIND_U64,
                    u64_value: value,
                    ..TkParameterValue::default()
                }),
            );
            apply_maximum(
                info,
                maximum.map(|value| TkParameterValue {
                    kind: TK_PARAMETER_KIND_U64,
                    u64_value: value,
                    ..TkParameterValue::default()
                }),
            );
        }
        ParameterType::F32 { minimum, maximum } => {
            apply_minimum(
                info,
                minimum.map(|value| TkParameterValue {
                    kind: TK_PARAMETER_KIND_F32,
                    f32_value: value,
                    ..TkParameterValue::default()
                }),
            );
            apply_maximum(
                info,
                maximum.map(|value| TkParameterValue {
                    kind: TK_PARAMETER_KIND_F32,
                    f32_value: value,
                    ..TkParameterValue::default()
                }),
            );
        }
        ParameterType::F64 { minimum, maximum } => {
            apply_minimum(
                info,
                minimum.map(|value| TkParameterValue {
                    kind: TK_PARAMETER_KIND_F64,
                    f64_value: value,
                    ..TkParameterValue::default()
                }),
            );
            apply_maximum(
                info,
                maximum.map(|value| TkParameterValue {
                    kind: TK_PARAMETER_KIND_F64,
                    f64_value: value,
                    ..TkParameterValue::default()
                }),
            );
        }
        ParameterType::Enum { options } => {
            info.enum_option_count = options.len();
        }
        ParameterType::Bool
        | ParameterType::Vector2F64
        | ParameterType::Vector3F64
        | ParameterType::String => {}
    }
}

fn apply_minimum(info: &mut TkParameterInfo, value: Option<TkParameterValue>) {
    if let Some(value) = value {
        info.has_minimum = TK_TRUE;
        info.minimum_value = value;
    }
}

fn apply_maximum(info: &mut TkParameterInfo, value: Option<TkParameterValue>) {
    if let Some(value) = value {
        info.has_maximum = TK_TRUE;
        info.maximum_value = value;
    }
}

fn enum_option_info(option: &EnumOption) -> TkEnumOptionInfo {
    TkEnumOptionInfo {
        id: TkStringView::from_str(option.id().as_str()),
        display_name: TkStringView::from_str(option.display_name()),
        description: TkStringView::from_str(option.description()),
    }
}
