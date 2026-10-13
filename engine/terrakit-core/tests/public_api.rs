use terrakit_core::{grid, prelude::*, region, terrain};

#[test]
fn root_exports_keep_core_primitives_easy_to_import() {
    let extent = Extent2::try_new(2, 2).unwrap();
    let field = Field2::filled(
        extent,
        7,
        GridTransform2::identity_xz(),
        SamplingDomain::Points,
    )
    .unwrap();

    assert_eq!(field.values(), &[7, 7, 7, 7]);
}

#[test]
fn domain_modules_expose_the_same_public_types() {
    let extent = grid::Extent2::try_new(2, 1).unwrap();
    let grid = grid::Grid2::filled(extent, 3).unwrap();
    let height_field = terrain::HeightField::from_values(
        extent,
        vec![1.0, 2.0],
        grid::GridTransform2::identity_xz(),
        grid::SamplingDomain::Points,
        Vector3F64::Y,
    )
    .unwrap();

    assert_eq!(grid.values(), &[3, 3]);
    assert_eq!(height_field.values(), &[1.0, 2.0]);
}

#[test]
fn prelude_is_enough_for_common_terrain_shapes() {
    let mesh = TerrainMesh::new(
        Vector3F64::ZERO,
        vec![
            Vector3F32::new(0.0, 0.0, 0.0),
            Vector3F32::new(1.0, 0.0, 0.0),
            Vector3F32::new(0.0, 0.0, 1.0),
        ],
        vec![0, 1, 2],
        Some(vec![Vector3F32::Y, Vector3F32::Y, Vector3F32::Y]),
        Some(vec![
            Vector2F32::ZERO,
            Vector2F32::new(1.0, 0.0),
            Vector2F32::new(0.0, 1.0),
        ]),
    )
    .unwrap();

    assert_eq!(mesh.triangle_count(), 1);
    assert_eq!(mesh.positions().len(), 3);
    assert_eq!(mesh.indices(), &[0, 1, 2]);
    assert_eq!(mesh.normals().unwrap(), &[Vector3F32::Y; 3]);
    assert_eq!(
        mesh.texcoords().unwrap(),
        &[
            Vector2F32::ZERO,
            Vector2F32::new(1.0, 0.0),
            Vector2F32::new(0.0, 1.0),
        ]
    );
}

#[test]
fn prelude_is_enough_for_mvp_volume_shapes() {
    let density = DensityField::from_dimensions(2, 2, 2).unwrap();
    let mut voxels = VoxelVolume::from_dimensions(2, 2, 2).unwrap();

    voxels.try_set(1, 1, 1, VoxelId::new(5)).unwrap();

    assert_eq!(density.values(), &[0.0; 8]);
    assert_eq!(voxels.get(1, 1, 1), Some(VoxelId::new(5)));
}

#[test]
fn prelude_exposes_canonical_seed_types() {
    let seed = GenerationSeed::new(123);
    let derived = seed.derive(SeedDomain::new(7));

    assert_eq!(seed.value(), 123);
    assert_eq!(derived, seed.derive(SeedDomain::new(7)));
}

#[test]
fn prelude_exposes_canonical_region_types() {
    let layout =
        RegionLayout2::new(Extent2::try_new(32, 32).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();
    let descriptor = layout
        .resolve(RegionRequest2::new(
            RegionCoord2::new(4, -2),
            LodLevel::HIGHEST,
        ))
        .unwrap();
    let generation_region = GenerationRegion::Region2(descriptor);

    assert_eq!(descriptor.coordinate(), RegionCoord2::new(4, -2));
    assert_eq!(
        descriptor.point_sample_extent(),
        Extent2::try_new(33, 33).unwrap()
    );
    assert_eq!(generation_region.lod(), LodLevel::HIGHEST);
    assert!(generation_region.as_region2().is_some());
}

#[test]
fn region_module_exposes_the_same_public_types() {
    let coord = region::RegionCoord3::new(-1, 2, 3);
    let layout = region::RegionLayout3::new(
        grid::Extent3::try_new(16, 32, 8).unwrap(),
        Vector3F64::new(1.0, 2.0, 1.0),
    )
    .unwrap();
    let descriptor = layout
        .resolve(region::RegionRequest3::new(coord, region::LodLevel::new(1)))
        .unwrap();

    assert_eq!(descriptor.coordinate(), coord);
    assert_eq!(
        descriptor.cell_extent(),
        grid::Extent3::try_new(8, 16, 4).unwrap()
    );
}
