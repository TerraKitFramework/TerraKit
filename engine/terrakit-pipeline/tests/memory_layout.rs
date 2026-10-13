use std::mem::{align_of, size_of};

use terrakit_pipeline::{ResourceKey, StageId, StageSchemaVersion};

#[test]
fn pipeline_transparent_wrappers_match_backing_integer_layout() {
    assert_same_layout::<ResourceKey, u64>();
    assert_same_layout::<StageId, u64>();
    assert_same_layout::<StageSchemaVersion, u32>();
}

fn assert_same_layout<T, U>() {
    assert_eq!(size_of::<T>(), size_of::<U>());
    assert_eq!(align_of::<T>(), align_of::<U>());
}
