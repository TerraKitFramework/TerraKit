mod common;

use common::*;
use terrakit::*;

#[test]
fn double_destroy_through_same_slot_is_safe() {
    let mut registry = create_registry();
    assert_ok(tk_stage_registry_destroy(&mut registry));
    assert!(registry.is_null());
    assert_ok(tk_stage_registry_destroy(&mut registry));
}

#[test]
fn results_outlive_runtime_and_multiple_results_are_independent() {
    let (mut registry, mut runtime) = build_runtime(false);
    let mut first = generate_region(runtime, 0, 0, 77);
    let mut second = generate_region(runtime, 1, 0, 77);

    assert_ok(tk_runtime_destroy(&mut runtime));
    assert!(runtime.is_null());

    let mut first_mesh = TkTerrainMeshView::default();
    let mut second_mesh = TkTerrainMeshView::default();
    assert_ok(tk_generation_result_get_mesh(
        first,
        TERRAIN_MESH,
        &mut first_mesh,
    ));
    assert_ok(tk_generation_result_get_mesh(
        second,
        TERRAIN_MESH,
        &mut second_mesh,
    ));
    assert_ne!(first_mesh.origin, second_mesh.origin);
    assert!(first_mesh.normals.is_null());
    assert_eq!(first_mesh.normal_count, 0);
    assert!(first_mesh.texcoords.is_null());
    assert_eq!(first_mesh.texcoord_count, 0);

    assert_ok(tk_generation_result_destroy(&mut first));
    assert_ok(tk_generation_result_destroy(&mut second));
    destroy_registry(&mut registry);
}
