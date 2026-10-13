//! Generation result metadata and immutable zero-copy resource views.

use terrakit_core::{
    DensityField, HeightField, RegionDescriptor2, RegionDescriptor3, ScatterPoints, TerrainMesh,
    VoxelVolume,
};
use terrakit_pipeline::{ResourceKey, ResourceSet};

use crate::{
    TkRegionKind, TkResourceKind, TkStatus,
    error::{AbiError, ffi_guard},
    ffi::{destroy_handle, out_ref, slice_pointer},
    runtime::{GenerationResultHandle, result_handle},
    types::{
        TkDensityFieldView, TkGenerationResult, TkGridTransform2, TkHeightFieldView, TkRegion2Info,
        TkRegion3Info, TkScatterPointsView, TkTerrainMeshView, TkVec2F64, TkVoxelVolumeView,
        region_kind_to_tk, resource_kind_to_tk, sampling_domain_to_tk,
    },
};

/// Destroys a generation result handle.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_destroy(result: *mut *mut TkGenerationResult) -> TkStatus {
    ffi_guard(|| destroy_handle::<GenerationResultHandle, TkGenerationResult>(result, "result"))
}

/// Returns the dimensionality of a generation result.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_region_kind(
    result: *const TkGenerationResult,
    out_kind: *mut TkRegionKind,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let out = out_ref(out_kind, "out_kind")?;
        *out = region_kind_to_tk(result.result.kind());
        Ok(())
    })
}

/// Returns the generation seed used for a result.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_seed(
    result: *const TkGenerationResult,
    out_seed: *mut u64,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let out = out_ref(out_seed, "out_seed")?;
        *out = result.result.seed().value();
        Ok(())
    })
}

/// Returns the opaque LOD tier used for a result.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_lod(
    result: *const TkGenerationResult,
    out_lod: *mut u16,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let out = out_ref(out_lod, "out_lod")?;
        *out = result.result.lod().value();
        Ok(())
    })
}

/// Returns resolved 2D region metadata for a 2D result.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_region_2d(
    result: *const TkGenerationResult,
    out_region: *mut TkRegion2Info,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let region = result
            .result
            .region2()
            .ok_or_else(|| AbiError::type_mismatch("generation result is not 2D"))?;
        let out = out_ref(out_region, "out_region")?;
        *out = TkRegion2Info::default();
        *out = region2_info(region);
        Ok(())
    })
}

/// Returns resolved 3D region metadata for a 3D result.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_region_3d(
    result: *const TkGenerationResult,
    out_region: *mut TkRegion3Info,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let region = result
            .result
            .region3()
            .ok_or_else(|| AbiError::type_mismatch("generation result is not 3D"))?;
        let out = out_ref(out_region, "out_region")?;
        *out = TkRegion3Info::default();
        *out = region3_info(region);
        Ok(())
    })
}

/// Returns the canonical resource kind for a generated resource key.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_resource_kind(
    result: *const TkGenerationResult,
    resource_key: u64,
    out_kind: *mut TkResourceKind,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let out = out_ref(out_kind, "out_kind")?;
        let key = ResourceKey(resource_key);
        let kind = result
            .result
            .resources()
            .kind(key)
            .ok_or_else(|| missing_resource(key))?;
        *out = resource_kind_to_tk(kind);
        Ok(())
    })
}

/// Returns a borrowed immutable height-field view for a generated resource.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_height_field(
    result: *const TkGenerationResult,
    resource_key: u64,
    out_view: *mut TkHeightFieldView,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let key = ResourceKey(resource_key);
        let height_field =
            result.result.resources().height_field(key).ok_or_else(|| {
                missing_or_wrong_kind(result.result.resources(), key, "height field")
            })?;
        let out = out_ref(out_view, "out_view")?;
        *out = TkHeightFieldView::default();
        *out = height_field_view(height_field);
        Ok(())
    })
}

/// Returns a borrowed immutable scatter-point view for a generated resource.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_scatter_points(
    result: *const TkGenerationResult,
    resource_key: u64,
    out_view: *mut TkScatterPointsView,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let key = ResourceKey(resource_key);

        let points = result
            .result
            .resources()
            .scatter_points(key)
            .ok_or_else(|| {
                missing_or_wrong_kind(result.result.resources(), key, "scatter points")
            })?;

        let out = out_ref(out_view, "out_view")?;

        *out = TkScatterPointsView::default();
        *out = scatter_points_view(points);

        Ok(())
    })
}

/// Returns a borrowed immutable density-field view for a generated resource.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_density_field(
    result: *const TkGenerationResult,
    resource_key: u64,
    out_view: *mut TkDensityFieldView,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let key = ResourceKey(resource_key);
        let density = result
            .result
            .resources()
            .density_field(key)
            .ok_or_else(|| {
                missing_or_wrong_kind(result.result.resources(), key, "density field")
            })?;
        let out = out_ref(out_view, "out_view")?;
        *out = TkDensityFieldView::default();
        *out = density_field_view(density);
        Ok(())
    })
}

/// Returns a borrowed immutable voxel-volume view for a generated resource.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_voxel_volume(
    result: *const TkGenerationResult,
    resource_key: u64,
    out_view: *mut TkVoxelVolumeView,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let key = ResourceKey(resource_key);
        let volume =
            result.result.resources().voxel_volume(key).ok_or_else(|| {
                missing_or_wrong_kind(result.result.resources(), key, "voxel volume")
            })?;
        let out = out_ref(out_view, "out_view")?;
        *out = TkVoxelVolumeView::default();
        *out = voxel_volume_view(volume);
        Ok(())
    })
}

/// Returns a borrowed immutable terrain-mesh view for a generated resource.
#[unsafe(no_mangle)]
pub extern "C" fn tk_generation_result_get_mesh(
    result: *const TkGenerationResult,
    resource_key: u64,
    out_view: *mut TkTerrainMeshView,
) -> TkStatus {
    ffi_guard(|| {
        let result = result_handle(result)?;
        let key = ResourceKey(resource_key);
        let mesh =
            result.result.resources().mesh(key).ok_or_else(|| {
                missing_or_wrong_kind(result.result.resources(), key, "terrain mesh")
            })?;
        let out = out_ref(out_view, "out_view")?;
        *out = TkTerrainMeshView::default();
        *out = mesh_view(mesh);
        Ok(())
    })
}

fn region2_info(region: &RegionDescriptor2) -> TkRegion2Info {
    let coordinate = region.coordinate();
    let cell = region.cell_extent();
    let point = region.point_sample_extent();
    TkRegion2Info {
        region_x: coordinate.x,
        region_y: coordinate.y,
        lod_level: region.lod().value(),
        reserved: [0; 6],
        cell_width: cell.width(),
        cell_height: cell.height(),
        point_width: point.width(),
        point_height: point.height(),
        effective_spacing: TkVec2F64::from(region.effective_spacing()),
        transform: TkGridTransform2::from(region.transform()),
    }
}

fn region3_info(region: &RegionDescriptor3) -> TkRegion3Info {
    let coordinate = region.coordinate();
    let cell = region.cell_extent();
    let point = region.point_sample_extent();
    TkRegion3Info {
        region_x: coordinate.x,
        region_y: coordinate.y,
        region_z: coordinate.z,
        lod_level: region.lod().value(),
        reserved: [0; 6],
        cell_width: cell.width(),
        cell_height: cell.height(),
        cell_depth: cell.depth(),
        reserved_extent: 0,
        point_width: point.width(),
        point_height: point.height(),
        point_depth: point.depth(),
        reserved_point_extent: 0,
        effective_spacing: region.effective_spacing().into(),
        transform: region.transform().into(),
    }
}

fn height_field_view(field: &HeightField) -> TkHeightFieldView {
    let extent = field.extent();
    TkHeightFieldView {
        width: extent.width(),
        height: extent.height(),
        sampling: sampling_domain_to_tk(field.sampling()),
        reserved: 0,
        transform: field.transform().into(),
        height_axis: field.height_axis().into(),
        values: slice_pointer(field.values()),
        value_count: field.len(),
    }
}

fn scatter_points_view(points: &ScatterPoints) -> TkScatterPointsView {
    TkScatterPointsView {
        origin: points.origin().into(),
        points: slice_pointer(points.points()).cast(),
        point_count: points.len(),
    }
}

fn density_field_view(field: &DensityField) -> TkDensityFieldView {
    let extent = field.extent();
    TkDensityFieldView {
        width: extent.width(),
        height: extent.height(),
        depth: extent.depth(),
        reserved_extent: 0,
        sampling: sampling_domain_to_tk(field.field().sampling()),
        reserved_sampling: 0,
        transform: field.field().transform().into(),
        values: slice_pointer(field.values()),
        value_count: field.len(),
    }
}

fn voxel_volume_view(volume: &VoxelVolume) -> TkVoxelVolumeView {
    let extent = volume.extent();
    TkVoxelVolumeView {
        width: extent.width(),
        height: extent.height(),
        depth: extent.depth(),
        reserved_extent: 0,
        sampling: sampling_domain_to_tk(volume.field().sampling()),
        reserved_sampling: 0,
        transform: volume.field().transform().into(),
        values: slice_pointer(volume.values()).cast::<u32>(),
        value_count: volume.len(),
    }
}

fn mesh_view(mesh: &TerrainMesh) -> TkTerrainMeshView {
    let (normals, normal_count) = optional_vec3_slice(mesh.normals());
    let (texcoords, texcoord_count) = optional_vec2_slice(mesh.texcoords());

    TkTerrainMeshView {
        origin: mesh.origin().into(),
        positions: slice_pointer(mesh.positions()).cast(),
        position_count: mesh.positions().len(),
        indices: slice_pointer(mesh.indices()),
        index_count: mesh.indices().len(),
        normals,
        normal_count,
        texcoords,
        texcoord_count,
    }
}

fn optional_vec3_slice(
    slice: Option<&[terrakit_core::Vector3F32]>,
) -> (*const crate::types::TkVec3F32, usize) {
    match slice {
        Some(slice) => (slice_pointer(slice).cast(), slice.len()),
        None => (std::ptr::null(), 0),
    }
}

fn optional_vec2_slice(
    slice: Option<&[terrakit_core::Vector2F32]>,
) -> (*const crate::types::TkVec2F32, usize) {
    match slice {
        Some(slice) => (slice_pointer(slice).cast(), slice.len()),
        None => (std::ptr::null(), 0),
    }
}

fn missing_or_wrong_kind(
    resources: &ResourceSet,
    key: ResourceKey,
    expected: &'static str,
) -> AbiError {
    match resources.kind(key) {
        Some(actual) => {
            AbiError::type_mismatch(format!("resource {} is {actual:?}, not {expected}", key.0))
        }
        None => missing_resource(key),
    }
}

fn missing_resource(key: ResourceKey) -> AbiError {
    AbiError::not_found(format!("resource {} was not found", key.0))
}
