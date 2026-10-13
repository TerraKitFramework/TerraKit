#![allow(dead_code)]

use std::{ffi::c_char, ptr, slice, str};

use terrakit::*;

pub const BASE_HEIGHT: u64 = 100;
pub const NOISY_HEIGHT: u64 = 101;
pub const TERRAIN_MESH: u64 = 102;

pub fn sv(value: &str) -> TkStringView {
    TkStringView {
        data: value.as_ptr().cast::<c_char>(),
        length: value.len(),
    }
}

pub fn view_text(view: TkStringView) -> String {
    if view.length == 0 {
        return String::new();
    }
    assert!(!view.data.is_null());
    // SAFETY: test callers only pass views returned from live TerraKit handles.
    let bytes = unsafe { slice::from_raw_parts(view.data.cast::<u8>(), view.length) };
    str::from_utf8(bytes).unwrap().to_owned()
}

pub fn last_error() -> String {
    let mut required = 0;
    assert_eq!(
        // SAFETY: querying accepts a null buffer with zero capacity, and
        // required points to writable test-owned storage.
        unsafe { tk_last_error_message_copy(ptr::null_mut(), 0, &mut required) },
        TK_STATUS_OK
    );
    let mut buffer = vec![0_u8; required];
    assert_eq!(
        // SAFETY: buffer is writable for buffer.len() bytes, and required
        // points to writable test-owned storage.
        unsafe {
            tk_last_error_message_copy(buffer.as_mut_ptr().cast(), buffer.len(), &mut required)
        },
        TK_STATUS_OK
    );
    String::from_utf8(buffer[..required.saturating_sub(1)].to_vec()).unwrap()
}

pub fn assert_ok(status: TkStatus) {
    assert_eq!(status, TK_STATUS_OK, "{}", last_error());
}

pub fn create_registry() -> *mut TkStageRegistry {
    let mut registry = ptr::null_mut();
    assert_ok(tk_stage_registry_create_builtin(&mut registry));
    assert!(!registry.is_null());
    registry
}

pub fn destroy_registry(registry: &mut *mut TkStageRegistry) {
    assert_ok(tk_stage_registry_destroy(registry));
    assert!(registry.is_null());
}

pub fn find_schema(registry: *const TkStageRegistry, stage_type: &str) -> usize {
    let mut index = usize::MAX;
    assert_ok(tk_stage_registry_find_schema(
        registry,
        sv(stage_type),
        &mut index,
    ));
    index
}

pub fn schema_info(registry: *const TkStageRegistry, stage_type: &str) -> TkStageSchemaInfo {
    let index = find_schema(registry, stage_type);
    let mut info = TkStageSchemaInfo::default();
    assert_ok(tk_stage_registry_get_schema(registry, index, &mut info));
    info
}

pub fn create_stage(stage_id: u64, stage_type: &str, version: u32) -> *mut TkStageConstruction {
    let mut construction = ptr::null_mut();
    assert_ok(tk_stage_construction_create(
        stage_id,
        sv(stage_type),
        version,
        &mut construction,
    ));
    assert!(!construction.is_null());
    construction
}

pub fn param_bool(value: bool) -> TkParameterValue {
    TkParameterValue {
        kind: TK_PARAMETER_KIND_BOOL,
        bool_value: if value { TK_TRUE } else { TK_FALSE },
        ..TkParameterValue::default()
    }
}

pub fn param_u64(value: u64) -> TkParameterValue {
    TkParameterValue {
        kind: TK_PARAMETER_KIND_U64,
        u64_value: value,
        ..TkParameterValue::default()
    }
}

pub fn param_f32(value: f32) -> TkParameterValue {
    TkParameterValue {
        kind: TK_PARAMETER_KIND_F32,
        f32_value: value,
        ..TkParameterValue::default()
    }
}

pub fn param_f64(value: f64) -> TkParameterValue {
    TkParameterValue {
        kind: TK_PARAMETER_KIND_F64,
        f64_value: value,
        ..TkParameterValue::default()
    }
}

pub fn param_enum(value: &str) -> TkParameterValue {
    TkParameterValue {
        kind: TK_PARAMETER_KIND_ENUM,
        text_value: sv(value),
        ..TkParameterValue::default()
    }
}

pub fn set_param(construction: *mut TkStageConstruction, id: &str, value: &TkParameterValue) {
    assert_ok(tk_stage_construction_set_parameter(
        construction,
        sv(id),
        value,
    ));
}

pub fn build_runtime(
    generate_optional_mesh_buffers: bool,
) -> (*mut TkStageRegistry, *mut TkRuntime) {
    let registry = create_registry();
    let flat_info = schema_info(registry, "terrakit.height.flat");
    let noise_info = schema_info(registry, "terrakit.height.noise");
    let mesh_info = schema_info(registry, "terrakit.mesh.height_field");

    let mut flat = create_stage(1, "terrakit.height.flat", flat_info.schema_version);
    assert_ok(tk_stage_construction_bind_output(
        flat,
        sv("height"),
        BASE_HEIGHT,
    ));

    let mut noise = create_stage(2, "terrakit.height.noise", noise_info.schema_version);
    assert_ok(tk_stage_construction_bind_input(
        noise,
        sv("source"),
        BASE_HEIGHT,
    ));
    assert_ok(tk_stage_construction_bind_output(
        noise,
        sv("height"),
        NOISY_HEIGHT,
    ));
    set_param(noise, "octaves", &param_u64(3));
    set_param(noise, "frequency", &param_f64(0.09));
    set_param(noise, "amplitude", &param_f32(0.25));
    set_param(noise, "normalize", &param_bool(true));
    set_param(noise, "mode", &param_enum("add"));

    let mut mesh = create_stage(3, "terrakit.mesh.height_field", mesh_info.schema_version);
    assert_ok(tk_stage_construction_bind_input(
        mesh,
        sv("height"),
        NOISY_HEIGHT,
    ));
    assert_ok(tk_stage_construction_bind_output(
        mesh,
        sv("mesh"),
        TERRAIN_MESH,
    ));
    if !generate_optional_mesh_buffers {
        set_param(mesh, "generate_normals", &param_bool(false));
        set_param(mesh, "generate_texcoords", &param_bool(false));
    }

    let mut assembler = ptr::null_mut();
    assert_ok(tk_pipeline_assembler_create(registry, &mut assembler));
    assert_ok(tk_pipeline_assembler_add_stage(assembler, &mut flat));
    assert!(flat.is_null());
    assert_ok(tk_pipeline_assembler_add_stage(assembler, &mut noise));
    assert!(noise.is_null());
    assert_ok(tk_pipeline_assembler_add_stage(assembler, &mut mesh));
    assert!(mesh.is_null());

    let mut pipeline = ptr::null_mut();
    assert_ok(tk_pipeline_assembler_finish(&mut assembler, &mut pipeline));
    assert!(assembler.is_null());
    assert!(!pipeline.is_null());

    let layout = TkRegionLayout2 {
        cell_width: 16,
        cell_height: 16,
        base_spacing: TkVec2F64 { x: 1.0, y: 1.0 },
    };
    let mut runtime = ptr::null_mut();
    assert_ok(tk_runtime_create_2d(&mut pipeline, &layout, &mut runtime));
    assert!(pipeline.is_null());
    assert!(!runtime.is_null());

    (registry, runtime)
}

pub fn generate_region(
    runtime: *mut TkRuntime,
    x: i64,
    y: i64,
    seed: u64,
) -> *mut TkGenerationResult {
    let request = TkGenerationRequest2 {
        seed,
        region_x: x,
        region_y: y,
        lod_level: 0,
        reserved: [0; 6],
    };
    let mut result = ptr::null_mut();
    assert_ok(tk_runtime_generate_2d(runtime, &request, &mut result));
    assert!(!result.is_null());
    result
}

pub fn height_values(view: &TkHeightFieldView) -> &[f32] {
    if view.value_count == 0 {
        return &[];
    }
    assert!(!view.values.is_null());
    // SAFETY: test views borrow from a live generation result kept alive by the caller.
    unsafe { slice::from_raw_parts(view.values, view.value_count) }
}

pub fn mesh_positions(view: &TkTerrainMeshView) -> &[TkVec3F32] {
    if view.position_count == 0 {
        return &[];
    }
    assert!(!view.positions.is_null());
    // SAFETY: test views borrow from a live generation result kept alive by the caller.
    unsafe { slice::from_raw_parts(view.positions, view.position_count) }
}

pub fn mesh_indices(view: &TkTerrainMeshView) -> &[u32] {
    if view.index_count == 0 {
        return &[];
    }
    assert!(!view.indices.is_null());
    // SAFETY: test views borrow from a live generation result kept alive by the caller.
    unsafe { slice::from_raw_parts(view.indices, view.index_count) }
}
