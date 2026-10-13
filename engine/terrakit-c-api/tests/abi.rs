mod common;

use std::ptr;

use common::*;
use terrakit::*;
use terrakit_builtins::{
    FlatHeightDefinition, HeightFieldMeshDefinition, NoiseHeightDefinition, builtin_stage_registry,
};
use terrakit_core::{Extent2, GenerationSeed, LodLevel, RegionCoord2, RegionLayout2, Vector2F64};
use terrakit_pipeline::{
    EnumValueId, ParameterId, ParameterSet, ParameterValue, PortId, ResourceKey, StageConstruction,
    StageId, StageTypeId, TerrainPipelineAssembler,
};
use terrakit_runtime::{GenerationRequest, TerrainRuntime};

#[test]
fn version_reporting_and_schema_discovery_work() {
    let mut version = TkVersion::default();
    assert_ok(tk_get_abi_version(&mut version));
    assert_eq!(
        version,
        TkVersion {
            major: 0,
            minor: 2,
            patch: 0
        }
    );

    let mut library_version = TkStringView::default();
    assert_ok(tk_get_library_version(&mut library_version));
    assert_eq!(view_text(library_version), env!("CARGO_PKG_VERSION"));

    let mut registry = create_registry();
    let mut count = 0;
    assert_ok(tk_stage_registry_get_schema_count(registry, &mut count));
    assert!(count > 0);

    destroy_registry(&mut registry);
}

#[test]
fn ports_parameters_defaults_bounds_and_enum_options_are_visible() {
    let mut registry = create_registry();
    let noise_index = find_schema(registry, "terrakit.height.noise");

    let mut input = TkInputPortInfo::default();
    assert_ok(tk_stage_registry_get_input_port(
        registry,
        noise_index,
        0,
        &mut input,
    ));
    assert_eq!(view_text(input.id), "source");
    assert_eq!(input.resource_kind, TK_RESOURCE_KIND_HEIGHT_FIELD);
    assert_eq!(input.optional, TK_FALSE);

    let mut output = TkOutputPortInfo::default();
    assert_ok(tk_stage_registry_get_output_port(
        registry,
        noise_index,
        0,
        &mut output,
    ));
    assert_eq!(view_text(output.id), "height");

    let mut algorithm = TkParameterInfo::default();
    assert_ok(tk_stage_registry_get_parameter(
        registry,
        noise_index,
        0,
        &mut algorithm,
    ));
    assert_eq!(view_text(algorithm.id), "algorithm");
    assert_eq!(algorithm.kind, TK_PARAMETER_KIND_ENUM);
    assert_eq!(algorithm.has_default, TK_TRUE);
    assert_eq!(view_text(algorithm.default_value.text_value), "value");
    assert_eq!(algorithm.enum_option_count, 4);

    let mut option = TkEnumOptionInfo::default();
    assert_ok(tk_stage_registry_get_enum_option(
        registry,
        noise_index,
        0,
        0,
        &mut option,
    ));
    assert_eq!(view_text(option.id), "value");

    assert_ok(tk_stage_registry_get_enum_option(
        registry,
        noise_index,
        0,
        1,
        &mut option,
    ));
    assert_eq!(view_text(option.id), "perlin");

    assert_ok(tk_stage_registry_get_enum_option(
        registry,
        noise_index,
        0,
        2,
        &mut option,
    ));
    assert_eq!(view_text(option.id), "simplex");

    assert_ok(tk_stage_registry_get_enum_option(
        registry,
        noise_index,
        0,
        3,
        &mut option,
    ));
    assert_eq!(view_text(option.id), "worley");

    let mut octaves = TkParameterInfo::default();
    assert_ok(tk_stage_registry_get_parameter(
        registry,
        noise_index,
        1,
        &mut octaves,
    ));
    assert_eq!(view_text(octaves.id), "octaves");
    assert_eq!(octaves.kind, TK_PARAMETER_KIND_U64);
    assert_eq!(octaves.has_minimum, TK_TRUE);
    assert_eq!(octaves.minimum_value.u64_value, 1);
    assert_eq!(octaves.has_maximum, TK_TRUE);
    assert!(octaves.maximum_value.u64_value >= 1);

    assert_eq!(
        tk_stage_registry_get_enum_option(registry, noise_index, 1, 0, &mut option),
        TK_STATUS_TYPE_MISMATCH
    );

    destroy_registry(&mut registry);
}

#[test]
fn invalid_arguments_are_rejected_before_rust_validation() {
    let mut construction = ptr::null_mut();
    let invalid_utf8 = [0xff_u8];
    let invalid_view = TkStringView {
        data: invalid_utf8.as_ptr().cast(),
        length: invalid_utf8.len(),
    };
    assert_eq!(
        tk_stage_construction_create(1, invalid_view, 1, &mut construction),
        TK_STATUS_INVALID_UTF8
    );

    assert_eq!(
        tk_stage_construction_create(1, TkStringView::default(), 1, &mut construction),
        TK_STATUS_INVALID_ARGUMENT
    );
    assert_eq!(
        tk_stage_construction_create(1, sv("terrakit.height.flat"), 0, &mut construction),
        TK_STATUS_INVALID_ARGUMENT
    );

    let mut valid = create_stage(1, "terrakit.height.flat", 1);
    let bad_bool = TkParameterValue {
        kind: TK_PARAMETER_KIND_BOOL,
        bool_value: 2,
        ..TkParameterValue::default()
    };
    assert_eq!(
        tk_stage_construction_set_parameter(valid, sv("elevation"), &bad_bool),
        TK_STATUS_INVALID_ARGUMENT
    );

    let bad_reserved = TkParameterValue {
        kind: TK_PARAMETER_KIND_F32,
        f32_value: 1.0,
        reserved_bool: [1, 0, 0],
        ..TkParameterValue::default()
    };
    assert_eq!(
        tk_stage_construction_set_parameter(valid, sv("elevation"), &bad_reserved),
        TK_STATUS_INVALID_ARGUMENT
    );

    let bad_kind = TkParameterValue {
        kind: 99,
        ..TkParameterValue::default()
    };
    assert_eq!(
        tk_stage_construction_set_parameter(valid, sv("elevation"), &bad_kind),
        TK_STATUS_INVALID_ARGUMENT
    );

    assert_ok(tk_stage_construction_destroy(&mut valid));
}

#[test]
fn construction_replacement_duplicate_bindings_and_assembly_errors_are_reported() {
    let mut registry = create_registry();
    let flat_info = schema_info(registry, "terrakit.height.flat");
    let mut flat = create_stage(1, "terrakit.height.flat", flat_info.schema_version);
    set_param(flat, "elevation", &param_f32(1.0));
    set_param(flat, "elevation", &param_f32(2.0));
    assert_ok(tk_stage_construction_bind_output(
        flat,
        sv("height"),
        BASE_HEIGHT,
    ));
    assert_eq!(
        tk_stage_construction_bind_output(flat, sv("height"), BASE_HEIGHT + 1),
        TK_STATUS_CONFIGURATION_ERROR
    );

    let mut unknown = create_stage(2, "terrakit.height.missing", 1);
    let mut assembler = ptr::null_mut();
    assert_ok(tk_pipeline_assembler_create(registry, &mut assembler));
    assert_eq!(
        tk_pipeline_assembler_add_stage(assembler, &mut unknown),
        TK_STATUS_NOT_FOUND
    );
    assert!(!unknown.is_null());
    assert_ok(tk_stage_construction_destroy(&mut unknown));

    let mut wrong_version = create_stage(3, "terrakit.height.flat", flat_info.schema_version + 1);
    assert_eq!(
        tk_pipeline_assembler_add_stage(assembler, &mut wrong_version),
        TK_STATUS_SCHEMA_VERSION_MISMATCH
    );
    assert_ok(tk_stage_construction_destroy(&mut wrong_version));
    assert_ok(tk_stage_construction_destroy(&mut flat));
    assert_ok(tk_pipeline_assembler_destroy(&mut assembler));
    destroy_registry(&mut registry);
}

#[test]
fn pipeline_runtime_generation_and_direct_rust_equivalence_work() {
    let (mut registry, mut runtime) = build_runtime(true);
    let mut result = generate_region(runtime, 0, 0, 1234);

    let mut kind = 0;
    assert_ok(tk_runtime_get_region_kind(runtime, &mut kind));
    assert_eq!(kind, TK_REGION_KIND_REGION_2);
    assert_ok(tk_generation_result_get_resource_kind(
        result,
        NOISY_HEIGHT,
        &mut kind,
    ));
    assert_eq!(kind, TK_RESOURCE_KIND_HEIGHT_FIELD);

    let mut height = TkHeightFieldView::default();
    assert_ok(tk_generation_result_get_height_field(
        result,
        NOISY_HEIGHT,
        &mut height,
    ));
    assert_eq!(height.width, 17);
    assert_eq!(height.height, 17);
    assert_eq!(height.value_count, 17 * 17);
    assert_eq!(height.sampling, TK_SAMPLING_DOMAIN_POINTS);

    let mut mesh = TkTerrainMeshView::default();
    assert_ok(tk_generation_result_get_mesh(
        result,
        TERRAIN_MESH,
        &mut mesh,
    ));
    assert_eq!(mesh.position_count, 17 * 17);
    assert_eq!(mesh.index_count % 3, 0);
    assert_eq!(mesh.normal_count, mesh.position_count);
    assert_eq!(mesh.texcoord_count, mesh.position_count);

    let direct = direct_rust_generation();
    let direct_height = direct
        .resources()
        .height_field(ResourceKey(NOISY_HEIGHT))
        .unwrap();
    assert_eq!(height_values(&height), direct_height.values());

    let direct_mesh = direct.resources().mesh(ResourceKey(TERRAIN_MESH)).unwrap();
    assert_eq!(mesh.origin, direct_mesh.origin().into());
    let direct_positions = direct_mesh
        .positions()
        .iter()
        .copied()
        .map(TkVec3F32::from)
        .collect::<Vec<_>>();
    assert_eq!(mesh_positions(&mesh), direct_positions.as_slice());
    assert_eq!(mesh_indices(&mesh), direct_mesh.indices());

    assert_ok(tk_generation_result_destroy(&mut result));
    assert_ok(tk_runtime_destroy(&mut runtime));
    destroy_registry(&mut registry);
}

fn direct_rust_generation() -> terrakit_runtime::GenerationResult {
    let registry = builtin_stage_registry().unwrap();
    let mut assembler = TerrainPipelineAssembler::new(&registry);
    assembler
        .add_stage(
            StageConstruction::new(
                StageId(1),
                StageTypeId::try_new(FlatHeightDefinition::TYPE_ID).unwrap(),
                FlatHeightDefinition::SCHEMA_VERSION,
            )
            .with_bindings(
                terrakit_pipeline::StageBindings::new()
                    .with_output(PortId::try_new("height").unwrap(), ResourceKey(BASE_HEIGHT))
                    .unwrap(),
            ),
        )
        .unwrap();

    let mut noise_parameters = ParameterSet::new();
    noise_parameters.set(
        ParameterId::try_new("octaves").unwrap(),
        ParameterValue::U64(3),
    );
    noise_parameters.set(
        ParameterId::try_new("frequency").unwrap(),
        ParameterValue::F64(0.09),
    );
    noise_parameters.set(
        ParameterId::try_new("amplitude").unwrap(),
        ParameterValue::F32(0.25),
    );
    noise_parameters.set(
        ParameterId::try_new("normalize").unwrap(),
        ParameterValue::Bool(true),
    );
    noise_parameters.set(
        ParameterId::try_new("mode").unwrap(),
        ParameterValue::Enum(EnumValueId::try_new("add").unwrap()),
    );
    assembler
        .add_stage(
            StageConstruction::new(
                StageId(2),
                StageTypeId::try_new(NoiseHeightDefinition::TYPE_ID).unwrap(),
                NoiseHeightDefinition::SCHEMA_VERSION,
            )
            .with_parameters(noise_parameters)
            .with_bindings(
                terrakit_pipeline::StageBindings::new()
                    .with_input(PortId::try_new("source").unwrap(), ResourceKey(BASE_HEIGHT))
                    .unwrap()
                    .with_output(
                        PortId::try_new("height").unwrap(),
                        ResourceKey(NOISY_HEIGHT),
                    )
                    .unwrap(),
            ),
        )
        .unwrap();

    assembler
        .add_stage(
            StageConstruction::new(
                StageId(3),
                StageTypeId::try_new(HeightFieldMeshDefinition::TYPE_ID).unwrap(),
                HeightFieldMeshDefinition::SCHEMA_VERSION,
            )
            .with_bindings(
                terrakit_pipeline::StageBindings::new()
                    .with_input(
                        PortId::try_new("height").unwrap(),
                        ResourceKey(NOISY_HEIGHT),
                    )
                    .unwrap()
                    .with_output(PortId::try_new("mesh").unwrap(), ResourceKey(TERRAIN_MESH))
                    .unwrap(),
            ),
        )
        .unwrap();

    let layout =
        RegionLayout2::new(Extent2::try_new(16, 16).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();
    let mut runtime = TerrainRuntime::new(layout, assembler.finish());
    runtime
        .generate(GenerationRequest::region2(
            GenerationSeed::new(1234),
            RegionCoord2::new(0, 0),
            LodLevel::HIGHEST,
        ))
        .unwrap()
}

#[test]
fn scatter_layout_matches_borrowed_c_view() {
    use std::mem::{align_of, offset_of, size_of};
    use terrakit_core::ScatterPoint;

    assert_eq!(size_of::<ScatterPoint>(), size_of::<TkScatterPoint>());
    assert_eq!(align_of::<ScatterPoint>(), align_of::<TkScatterPoint>());
    assert_eq!(
        offset_of!(ScatterPoint, position),
        offset_of!(TkScatterPoint, position)
    );
    assert_eq!(
        offset_of!(ScatterPoint, rotation),
        offset_of!(TkScatterPoint, rotation)
    );
    assert_eq!(
        offset_of!(ScatterPoint, scale),
        offset_of!(TkScatterPoint, scale)
    );
    assert_eq!(
        offset_of!(ScatterPoint, prototype_id),
        offset_of!(TkScatterPoint, prototype_id)
    );
}

#[test]
fn scatter_pipeline_returns_interpolated_borrowed_c_placements() {
    const SCATTER: u64 = 108;
    let mut registry = create_registry();
    let flat_info = schema_info(registry, "terrakit.height.flat");
    let scatter_info = schema_info(registry, "terrakit.scatter.poisson");
    let mut flat = create_stage(1, "terrakit.height.flat", flat_info.schema_version);
    set_param(flat, "elevation", &param_f32(8.0));
    assert_ok(tk_stage_construction_bind_output(
        flat,
        sv("height"),
        BASE_HEIGHT,
    ));

    let mut scatter = create_stage(2, "terrakit.scatter.poisson", scatter_info.schema_version);
    assert_ok(tk_stage_construction_bind_input(
        scatter,
        sv("height"),
        BASE_HEIGHT,
    ));
    assert_ok(tk_stage_construction_bind_output(
        scatter,
        sv("scatter"),
        SCATTER,
    ));
    set_param(scatter, "radius", &param_f32(2.0));
    set_param(scatter, "prototype_id", &param_u64(17));

    let mut assembler = ptr::null_mut();
    assert_ok(tk_pipeline_assembler_create(registry, &mut assembler));
    assert_ok(tk_pipeline_assembler_add_stage(assembler, &mut flat));
    assert_ok(tk_pipeline_assembler_add_stage(assembler, &mut scatter));

    let mut pipeline = ptr::null_mut();
    assert_ok(tk_pipeline_assembler_finish(&mut assembler, &mut pipeline));
    let layout = TkRegionLayout2 {
        cell_width: 16,
        cell_height: 16,
        base_spacing: TkVec2F64 { x: 1.0, y: 1.0 },
    };
    let mut runtime = ptr::null_mut();
    assert_ok(tk_runtime_create_2d(&mut pipeline, &layout, &mut runtime));
    let mut result = generate_region(runtime, 62_500_000, 0, 129);

    let mut kind = TK_RESOURCE_KIND_INVALID;
    assert_ok(tk_generation_result_get_resource_kind(
        result, SCATTER, &mut kind,
    ));
    assert_eq!(kind, TK_RESOURCE_KIND_SCATTER_POINTS);
    let mut view = TkScatterPointsView::default();
    assert_ok(tk_generation_result_get_scatter_points(
        result, SCATTER, &mut view,
    ));
    assert!(view.point_count > 0);
    assert!(!view.points.is_null());
    // SAFETY: The generated result owns the data and is still alive.
    let points = unsafe { std::slice::from_raw_parts(view.points, view.point_count) };
    assert!(
        points
            .iter()
            .all(|point| (point.position[1] - 8.0).abs() < 1e-5)
    );
    assert!(points.iter().all(|point| point.prototype_id == 17));
    assert_eq!(view.origin.x, 1_000_000_000.0);
    assert_eq!(view.origin.y, 0.0);

    assert_ok(tk_generation_result_destroy(&mut result));
    assert_ok(tk_runtime_destroy(&mut runtime));
    destroy_registry(&mut registry);
}
