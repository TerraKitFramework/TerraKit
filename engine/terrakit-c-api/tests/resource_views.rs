mod common;

use std::mem;

use common::*;
use terrakit::*;
use terrakit_core::{Vector2F32, Vector2F64, Vector3F32, Vector3F64, VoxelId};

#[test]
fn abi_vector_and_raw_resource_element_layouts_match_core() {
    assert_eq!(mem::size_of::<TkVec2F32>(), mem::size_of::<Vector2F32>());
    assert_eq!(mem::align_of::<TkVec2F32>(), mem::align_of::<Vector2F32>());
    assert_eq!(
        mem::offset_of!(TkVec2F32, x),
        mem::offset_of!(Vector2F32, x)
    );
    assert_eq!(
        mem::offset_of!(TkVec2F32, y),
        mem::offset_of!(Vector2F32, y)
    );

    assert_eq!(mem::size_of::<TkVec3F32>(), mem::size_of::<Vector3F32>());
    assert_eq!(mem::align_of::<TkVec3F32>(), mem::align_of::<Vector3F32>());
    assert_eq!(
        mem::offset_of!(TkVec3F32, z),
        mem::offset_of!(Vector3F32, z)
    );

    assert_eq!(mem::size_of::<TkVec2F64>(), mem::size_of::<Vector2F64>());
    assert_eq!(mem::align_of::<TkVec2F64>(), mem::align_of::<Vector2F64>());
    assert_eq!(
        mem::offset_of!(TkVec2F64, y),
        mem::offset_of!(Vector2F64, y)
    );

    assert_eq!(mem::size_of::<TkVec3F64>(), mem::size_of::<Vector3F64>());
    assert_eq!(mem::align_of::<TkVec3F64>(), mem::align_of::<Vector3F64>());
    assert_eq!(
        mem::offset_of!(TkVec3F64, z),
        mem::offset_of!(Vector3F64, z)
    );

    assert_eq!(
        mem::size_of::<u32>(),
        mem::size_of::<terrakit_core::MeshIndex>()
    );
    assert_eq!(
        mem::align_of::<u32>(),
        mem::align_of::<terrakit_core::MeshIndex>()
    );
    assert_eq!(mem::size_of::<u32>(), mem::size_of::<VoxelId>());
    assert_eq!(mem::align_of::<u32>(), mem::align_of::<VoxelId>());
}

#[test]
fn missing_and_wrong_resource_views_report_stable_statuses() {
    let (mut registry, mut runtime) = build_runtime(true);
    let mut result = generate_region(runtime, 0, 0, 5);

    let mut height = TkHeightFieldView::default();
    assert_eq!(
        tk_generation_result_get_height_field(result, TERRAIN_MESH, &mut height),
        TK_STATUS_TYPE_MISMATCH
    );
    assert_eq!(
        tk_generation_result_get_height_field(result, 999, &mut height),
        TK_STATUS_NOT_FOUND
    );

    let mut density = TkDensityFieldView::default();
    assert_eq!(
        tk_generation_result_get_density_field(result, NOISY_HEIGHT, &mut density),
        TK_STATUS_TYPE_MISMATCH
    );

    let mut voxel = TkVoxelVolumeView::default();
    assert_eq!(
        tk_generation_result_get_voxel_volume(result, NOISY_HEIGHT, &mut voxel),
        TK_STATUS_TYPE_MISMATCH
    );

    let mut region3 = TkRegion3Info::default();
    assert_eq!(
        tk_generation_result_get_region_3d(result, &mut region3),
        TK_STATUS_TYPE_MISMATCH
    );

    assert_ok(tk_generation_result_destroy(&mut result));
    assert_ok(tk_runtime_destroy(&mut runtime));
    destroy_registry(&mut registry);
}
