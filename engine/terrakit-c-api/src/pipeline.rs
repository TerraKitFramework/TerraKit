//! Pipeline assembler and completed pipeline handle operations.

use std::sync::Arc;

use terrakit_pipeline::{
    StageRegistry, TerrainPipeline, TerrainPipelineAssembler, ValidatedStageBindings,
};

use crate::{
    TkStatus,
    construction::StageConstructionHandle,
    error::{AbiError, assembly_error, ffi_guard, parameter_error},
    ffi::{destroy_handle, handle_slot, ref_from_ptr, take_handle},
    registry::registry_handle,
    types::{TkPipeline, TkPipelineAssembler, TkStageConstruction, TkStageRegistry},
};

/// Internal storage for a pipeline assembler handle.
pub(crate) struct PipelineAssemblerHandle {
    pub(crate) registry: Arc<StageRegistry>,
    stages: Vec<terrakit_pipeline::StageConstruction>,
}

/// Internal storage for a completed pipeline handle.
pub(crate) struct PipelineHandle {
    pub(crate) pipeline: TerrainPipeline,
}

/// Creates a pipeline assembler that retains the registry independently.
#[unsafe(no_mangle)]
pub extern "C" fn tk_pipeline_assembler_create(
    registry: *const TkStageRegistry,
    out_assembler: *mut *mut TkPipelineAssembler,
) -> TkStatus {
    ffi_guard(|| {
        let registry = registry_handle(registry)?;
        let out = handle_slot(out_assembler, "out_assembler")?;
        *out = std::ptr::null_mut();

        let handle = Box::new(PipelineAssemblerHandle {
            registry: Arc::clone(&registry.registry),
            stages: Vec::new(),
        });
        *out = Box::into_raw(handle).cast::<TkPipelineAssembler>();
        Ok(())
    })
}

/// Destroys a pipeline assembler handle.
#[unsafe(no_mangle)]
pub extern "C" fn tk_pipeline_assembler_destroy(
    assembler: *mut *mut TkPipelineAssembler,
) -> TkStatus {
    ffi_guard(|| {
        destroy_handle::<PipelineAssemblerHandle, TkPipelineAssembler>(assembler, "assembler")
    })
}

/// Validates and moves one construction into the assembler's ordered list.
#[unsafe(no_mangle)]
pub extern "C" fn tk_pipeline_assembler_add_stage(
    assembler: *mut TkPipelineAssembler,
    construction: *mut *mut TkStageConstruction,
) -> TkStatus {
    ffi_guard(|| {
        let assembler = crate::ffi::mut_from_ptr::<PipelineAssemblerHandle, TkPipelineAssembler>(
            assembler,
            "assembler",
        )?;
        let construction_slot = handle_slot(construction, "construction")?;
        let construction_ref = ref_from_ptr::<StageConstructionHandle, TkStageConstruction>(
            *construction_slot,
            "construction",
        )?;
        let stage = construction_ref.to_construction();

        validate_construction_against_registry(&assembler.registry, &stage)?;

        let construction = take_handle::<StageConstructionHandle, TkStageConstruction>(
            construction_slot,
            "construction",
        )?;
        assembler.stages.push(construction.to_construction());
        Ok(())
    })
}

/// Consumes an assembler and returns a completed pipeline.
#[unsafe(no_mangle)]
pub extern "C" fn tk_pipeline_assembler_finish(
    assembler: *mut *mut TkPipelineAssembler,
    out_pipeline: *mut *mut TkPipeline,
) -> TkStatus {
    ffi_guard(|| {
        let out = handle_slot(out_pipeline, "out_pipeline")?;
        *out = std::ptr::null_mut();

        let assembler_slot = handle_slot(assembler, "assembler")?;
        let assembler = take_handle::<PipelineAssemblerHandle, TkPipelineAssembler>(
            assembler_slot,
            "assembler",
        )?;

        let mut rust_assembler = TerrainPipelineAssembler::new(&assembler.registry);
        for stage in assembler.stages {
            rust_assembler.add_stage(stage).map_err(assembly_error)?;
        }

        let pipeline = rust_assembler.finish();
        let handle = Box::new(PipelineHandle { pipeline });
        *out = Box::into_raw(handle).cast::<TkPipeline>();
        Ok(())
    })
}

/// Destroys a completed pipeline handle.
#[unsafe(no_mangle)]
pub extern "C" fn tk_pipeline_destroy(pipeline: *mut *mut TkPipeline) -> TkStatus {
    ffi_guard(|| destroy_handle::<PipelineHandle, TkPipeline>(pipeline, "pipeline"))
}

pub(crate) fn pipeline_slot<'a>(
    pipeline: *mut *mut TkPipeline,
) -> Result<&'a mut *mut TkPipeline, AbiError> {
    handle_slot(pipeline, "pipeline")
}

fn validate_construction_against_registry(
    registry: &StageRegistry,
    construction: &terrakit_pipeline::StageConstruction,
) -> Result<(), AbiError> {
    let definition = registry
        .definition(construction.stage_type())
        .ok_or_else(|| {
            AbiError::not_found(format!(
                "stage type '{}' was not found in assembler registry",
                construction.stage_type()
            ))
        })?;
    let schema = definition.schema();

    if construction.schema_version() != schema.version() {
        return Err(AbiError::new(
            crate::TK_STATUS_SCHEMA_VERSION_MISMATCH,
            format!(
                "stage {} requested schema version {} for '{}', but version {} is registered",
                construction.stage_id().0,
                construction.schema_version(),
                construction.stage_type(),
                schema.version()
            ),
        ));
    }

    schema
        .resolve_parameters(construction.parameters())
        .map_err(|error| parameter_error("failed to resolve stage parameters", error))?;
    ValidatedStageBindings::validate(schema, construction.bindings())
        .map_err(|error| crate::error::binding_error("failed to validate stage bindings", error))?;

    Ok(())
}
