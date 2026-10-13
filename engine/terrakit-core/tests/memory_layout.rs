use std::mem::{align_of, offset_of, size_of};

use terrakit_core::{
    GenerationSeed, LodLevel, SeedDomain, Vector2F32, Vector2F64, Vector3F32, Vector3F64, VoxelId,
};

#[test]
fn vector2_f32_layout_is_stable() {
    assert_eq!(offset_of!(Vector2F32, x), 0);
    assert_eq!(offset_of!(Vector2F32, y), 4);
    assert_eq!(size_of::<Vector2F32>(), 8);
    assert_eq!(align_of::<Vector2F32>(), align_of::<f32>());
}

#[test]
fn vector3_f32_layout_is_stable() {
    assert_eq!(offset_of!(Vector3F32, x), 0);
    assert_eq!(offset_of!(Vector3F32, y), 4);
    assert_eq!(offset_of!(Vector3F32, z), 8);
    assert_eq!(size_of::<Vector3F32>(), 12);
    assert_eq!(align_of::<Vector3F32>(), align_of::<f32>());
}

#[test]
fn vector2_f64_layout_is_stable() {
    assert_eq!(offset_of!(Vector2F64, x), 0);
    assert_eq!(offset_of!(Vector2F64, y), 8);
    assert_eq!(size_of::<Vector2F64>(), 16);
    assert_eq!(align_of::<Vector2F64>(), align_of::<f64>());
}

#[test]
fn vector3_f64_layout_is_stable() {
    assert_eq!(offset_of!(Vector3F64, x), 0);
    assert_eq!(offset_of!(Vector3F64, y), 8);
    assert_eq!(offset_of!(Vector3F64, z), 16);
    assert_eq!(size_of::<Vector3F64>(), 24);
    assert_eq!(align_of::<Vector3F64>(), align_of::<f64>());
}

#[test]
fn core_transparent_wrappers_match_backing_integer_layout() {
    assert_same_layout::<GenerationSeed, u64>();
    assert_same_layout::<SeedDomain, u64>();
    assert_same_layout::<LodLevel, u16>();
    assert_same_layout::<VoxelId, u32>();
}

fn assert_same_layout<T, U>() {
    assert_eq!(size_of::<T>(), size_of::<U>());
    assert_eq!(align_of::<T>(), align_of::<U>());
}
