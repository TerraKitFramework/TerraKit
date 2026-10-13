//! Runtime handle creation, dimensionality query, generation, and destruction.

use terrakit_core::{
    Extent2, Extent3, GenerationSeed, LodLevel, RegionCoord2, RegionCoord3, RegionLayout2,
    RegionLayout3, Vector2F64, Vector3F64,
};
use terrakit_runtime::{GenerationRequest, TerrainRuntime};

use crate::{
    TkRegionKind, TkStatus,
    error::{AbiError, ffi_guard, runtime_error},
    ffi::{
        destroy_handle, handle_slot, mut_from_ptr, out_ref, ref_from_ptr, require_zero_reserved,
    },
    pipeline::{PipelineHandle, pipeline_slot},
    types::{
        TkGenerationRequest2, TkGenerationRequest3, TkGenerationResult, TkPipeline,
        TkRegionLayout2, TkRegionLayout3, TkRuntime, region_kind_to_tk,
    },
};

/// Internal storage for a runtime handle.
pub(crate) struct RuntimeHandle {
    pub(crate) runtime: TerrainRuntime,
}

/// Internal storage for a generation result handle.
pub(crate) struct GenerationResultHandle {
    pub(crate) result: terrakit_runtime::GenerationResult,
}

/// Creates a synchronous 2D runtime, consuming the pipeline on success.
#[unsafe(no_mangle)]
pub extern "C" fn tk_runtime_create_2d(
    pipeline: *mut *mut TkPipeline,
    layout: *const TkRegionLayout2,
    out_runtime: *mut *mut TkRuntime,
) -> TkStatus {
    ffi_guard(|| {
        let out = handle_slot(out_runtime, "out_runtime")?;
        *out = std::ptr::null_mut();

        let layout = ref_from_ptr::<TkRegionLayout2, TkRegionLayout2>(layout, "layout")?;
        let layout = region_layout_2d(*layout)?;
        let pipeline_slot = pipeline_slot(pipeline)?;

        let pipeline =
            crate::ffi::take_handle::<PipelineHandle, TkPipeline>(pipeline_slot, "pipeline")?;
        let runtime = TerrainRuntime::new(layout, pipeline.pipeline);
        let handle = Box::new(RuntimeHandle { runtime });
        *out = Box::into_raw(handle).cast::<TkRuntime>();
        Ok(())
    })
}

/// Creates a synchronous 3D runtime, consuming the pipeline on success.
#[unsafe(no_mangle)]
pub extern "C" fn tk_runtime_create_3d(
    pipeline: *mut *mut TkPipeline,
    layout: *const TkRegionLayout3,
    out_runtime: *mut *mut TkRuntime,
) -> TkStatus {
    ffi_guard(|| {
        let out = handle_slot(out_runtime, "out_runtime")?;
        *out = std::ptr::null_mut();

        let layout = ref_from_ptr::<TkRegionLayout3, TkRegionLayout3>(layout, "layout")?;
        let layout = region_layout_3d(*layout)?;
        let pipeline_slot = pipeline_slot(pipeline)?;

        let pipeline =
            crate::ffi::take_handle::<PipelineHandle, TkPipeline>(pipeline_slot, "pipeline")?;
        let runtime = TerrainRuntime::new(layout, pipeline.pipeline);
        let handle = Box::new(RuntimeHandle { runtime });
        *out = Box::into_raw(handle).cast::<TkRuntime>();
        Ok(())
    })
}

/// Returns the dimensionality of a runtime.
#[unsafe(no_mangle)]
pub extern "C" fn tk_runtime_get_region_kind(
    runtime: *const TkRuntime,
    out_kind: *mut TkRegionKind,
) -> TkStatus {
    ffi_guard(|| {
        let runtime = runtime_ref(runtime)?;
        let out = out_ref(out_kind, "out_kind")?;
        *out = region_kind_to_tk(runtime.runtime.layout().kind());
        Ok(())
    })
}

/// Runs one synchronous 2D generation request.
#[unsafe(no_mangle)]
pub extern "C" fn tk_runtime_generate_2d(
    runtime: *mut TkRuntime,
    request: *const TkGenerationRequest2,
    out_result: *mut *mut TkGenerationResult,
) -> TkStatus {
    ffi_guard(|| {
        let out = handle_slot(out_result, "out_result")?;
        *out = std::ptr::null_mut();
        let runtime = runtime_handle(runtime)?;
        let request =
            ref_from_ptr::<TkGenerationRequest2, TkGenerationRequest2>(request, "request")?;
        require_zero_reserved(&request.reserved, "request.reserved")?;

        let request = GenerationRequest::region2(
            GenerationSeed::new(request.seed),
            RegionCoord2::new(request.region_x, request.region_y),
            LodLevel::new(request.lod_level),
        );
        let result = runtime.runtime.generate(request).map_err(runtime_error)?;
        let handle = Box::new(GenerationResultHandle { result });
        *out = Box::into_raw(handle).cast::<TkGenerationResult>();
        Ok(())
    })
}

/// Runs one synchronous 3D generation request.
#[unsafe(no_mangle)]
pub extern "C" fn tk_runtime_generate_3d(
    runtime: *mut TkRuntime,
    request: *const TkGenerationRequest3,
    out_result: *mut *mut TkGenerationResult,
) -> TkStatus {
    ffi_guard(|| {
        let out = handle_slot(out_result, "out_result")?;
        *out = std::ptr::null_mut();
        let runtime = runtime_handle(runtime)?;
        let request =
            ref_from_ptr::<TkGenerationRequest3, TkGenerationRequest3>(request, "request")?;
        require_zero_reserved(&request.reserved, "request.reserved")?;

        let request = GenerationRequest::region3(
            GenerationSeed::new(request.seed),
            RegionCoord3::new(request.region_x, request.region_y, request.region_z),
            LodLevel::new(request.lod_level),
        );
        let result = runtime.runtime.generate(request).map_err(runtime_error)?;
        let handle = Box::new(GenerationResultHandle { result });
        *out = Box::into_raw(handle).cast::<TkGenerationResult>();
        Ok(())
    })
}

/// Destroys a runtime handle.
#[unsafe(no_mangle)]
pub extern "C" fn tk_runtime_destroy(runtime: *mut *mut TkRuntime) -> TkStatus {
    ffi_guard(|| destroy_handle::<RuntimeHandle, TkRuntime>(runtime, "runtime"))
}

pub(crate) fn runtime_handle<'a>(
    runtime: *mut TkRuntime,
) -> Result<&'a mut RuntimeHandle, AbiError> {
    mut_from_ptr::<RuntimeHandle, TkRuntime>(runtime, "runtime")
}

fn runtime_ref<'a>(runtime: *const TkRuntime) -> Result<&'a RuntimeHandle, AbiError> {
    ref_from_ptr::<RuntimeHandle, TkRuntime>(runtime, "runtime")
}

pub(crate) fn result_handle<'a>(
    result: *const TkGenerationResult,
) -> Result<&'a GenerationResultHandle, AbiError> {
    ref_from_ptr::<GenerationResultHandle, TkGenerationResult>(result, "result")
}

fn region_layout_2d(layout: TkRegionLayout2) -> Result<RegionLayout2, AbiError> {
    let extent = Extent2::try_new(layout.cell_width, layout.cell_height).map_err(|error| {
        AbiError::invalid_argument(format!("invalid 2D region layout: {error}"))
    })?;
    RegionLayout2::new(extent, Vector2F64::from(layout.base_spacing))
        .map_err(|error| AbiError::invalid_argument(format!("invalid 2D region layout: {error}")))
}

fn region_layout_3d(layout: TkRegionLayout3) -> Result<RegionLayout3, AbiError> {
    if layout.reserved != 0 {
        return Err(AbiError::invalid_argument(
            "reserved field 'layout.reserved' must be zero",
        ));
    }
    let extent = Extent3::try_new(layout.cell_width, layout.cell_height, layout.cell_depth)
        .map_err(|error| {
            AbiError::invalid_argument(format!("invalid 3D region layout: {error}"))
        })?;
    RegionLayout3::new(extent, Vector3F64::from(layout.base_spacing))
        .map_err(|error| AbiError::invalid_argument(format!("invalid 3D region layout: {error}")))
}
