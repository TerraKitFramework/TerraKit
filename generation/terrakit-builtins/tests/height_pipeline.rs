use terrakit_algorithms::noise::{FractalSettings, NoiseAlgorithm};
use terrakit_builtins::{FlatHeightStage, HeightFieldMeshStage, HeightNoiseMode, NoiseHeightStage};
use terrakit_core::{
    Extent2, GenerationRegion, LodLevel, RegionCoord2, RegionLayout2, RegionRequest2, SeedDomain,
    TerrainMesh, Vector2F64, Vector3F64,
};
use terrakit_pipeline::{
    ResourceKey, ResourceKind, ResourceSet, StageContext, StageId, TerrainPipeline,
};

const BASE_HEIGHT: ResourceKey = ResourceKey(100);
const NOISY_HEIGHT: ResourceKey = ResourceKey(101);
const TERRAIN_MESH: ResourceKey = ResourceKey(102);

fn settings() -> FractalSettings {
    FractalSettings {
        octaves: 4,
        frequency: 0.21,
        lacunarity: 2.0,
        persistence: 0.5,
        amplitude: 0.8,
        normalize: true,
    }
}

fn run_height_mesh_pipeline(seed: u64, coordinate: RegionCoord2) -> ResourceSet {
    let layout =
        RegionLayout2::new(Extent2::try_new(10, 7).unwrap(), Vector2F64::new(1.0, 1.0)).unwrap();
    let region = GenerationRegion::Region2(
        layout
            .resolve(RegionRequest2::new(coordinate, LodLevel::HIGHEST))
            .unwrap(),
    );
    let mut pipeline = TerrainPipeline::new();

    pipeline.add_stage(FlatHeightStage::standard(StageId(1), BASE_HEIGHT, 0.0).unwrap());
    pipeline.add_stage(
        NoiseHeightStage::new(
            StageId(2),
            BASE_HEIGHT,
            NOISY_HEIGHT,
            NoiseAlgorithm::Value,
            settings(),
            SeedDomain::new(99),
            HeightNoiseMode::Add,
        )
        .unwrap(),
    );
    pipeline.add_stage(HeightFieldMeshStage::new(
        StageId(3),
        NOISY_HEIGHT,
        TERRAIN_MESH,
    ));

    let mut resources = ResourceSet::new();
    pipeline
        .execute(&StageContext::from_u64(seed, region), &mut resources)
        .unwrap();

    resources
}

#[test]
fn flat_then_noise_then_mesh_pipeline_produces_deterministic_resources() {
    let first = run_height_mesh_pipeline(1234, RegionCoord2::ZERO);
    let second = run_height_mesh_pipeline(1234, RegionCoord2::ZERO);

    assert_eq!(first.kind(BASE_HEIGHT), Some(ResourceKind::HeightField));
    assert_eq!(first.kind(NOISY_HEIGHT), Some(ResourceKind::HeightField));
    assert_eq!(first.kind(TERRAIN_MESH), Some(ResourceKind::Mesh));

    let first_base = first.height_field(BASE_HEIGHT).unwrap();
    let first_height = first.height_field(NOISY_HEIGHT).unwrap();
    let second_height = second.height_field(NOISY_HEIGHT).unwrap();
    let first_mesh = first.mesh(TERRAIN_MESH).unwrap();
    let second_mesh = second.mesh(TERRAIN_MESH).unwrap();

    assert!(first_base.values().iter().all(|value| *value == 0.0));
    assert_eq!(first_height.extent(), Extent2::try_new(11, 8).unwrap());
    assert!(first_height.values().iter().all(|value| value.is_finite()));
    assert!(first_height.values().iter().any(|value| *value != 0.0));
    assert_eq!(first_height.values(), second_height.values());

    assert_eq!(first_mesh.vertex_count(), 88);
    assert_eq!(first_mesh.index_count(), 420);
    assert_eq!(first_mesh.triangle_count(), 140);
    assert!(
        first_mesh
            .positions()
            .iter()
            .all(|position| position.is_finite())
    );
    assert!(first_mesh.indices().iter().all(|index| *index < 88));
    assert!(
        first_mesh
            .normals()
            .unwrap()
            .iter()
            .all(|normal| normal.is_finite())
    );
    assert!(first_mesh.texcoords().unwrap().iter().all(|texcoord| {
        texcoord.is_finite()
            && (0.0..=1.0).contains(&texcoord.x)
            && (0.0..=1.0).contains(&texcoord.y)
    }));
    assert_eq!(first_mesh, second_mesh);
}

#[test]
fn adjacent_region_mesh_edges_reconstruct_matching_world_positions() {
    let left = run_height_mesh_pipeline(1234, RegionCoord2::new(-1, 0));
    let center = run_height_mesh_pipeline(1234, RegionCoord2::ZERO);
    let right = run_height_mesh_pipeline(1234, RegionCoord2::new(1, 0));
    let negative_z = run_height_mesh_pipeline(1234, RegionCoord2::new(0, -1));
    let positive_z = run_height_mesh_pipeline(1234, RegionCoord2::new(0, 1));

    assert_right_edge_matches_left_edge(
        left.mesh(TERRAIN_MESH).unwrap(),
        center.mesh(TERRAIN_MESH).unwrap(),
        11,
        8,
    );
    assert_right_edge_matches_left_edge(
        center.mesh(TERRAIN_MESH).unwrap(),
        right.mesh(TERRAIN_MESH).unwrap(),
        11,
        8,
    );
    assert_positive_z_edge_matches_negative_z_edge(
        negative_z.mesh(TERRAIN_MESH).unwrap(),
        center.mesh(TERRAIN_MESH).unwrap(),
        11,
        8,
    );
    assert_positive_z_edge_matches_negative_z_edge(
        center.mesh(TERRAIN_MESH).unwrap(),
        positive_z.mesh(TERRAIN_MESH).unwrap(),
        11,
        8,
    );

    let left_height = left.height_field(NOISY_HEIGHT).unwrap();
    let center_height = center.height_field(NOISY_HEIGHT).unwrap();
    let right_height = right.height_field(NOISY_HEIGHT).unwrap();
    let negative_z_height = negative_z.height_field(NOISY_HEIGHT).unwrap();
    let positive_z_height = positive_z.height_field(NOISY_HEIGHT).unwrap();
    let edge_x = left_height.width() - 1;
    let edge_y = negative_z_height.height() - 1;

    for y in 0..left_height.height() {
        assert_eq!(left_height.get(edge_x, y), center_height.get(0, y));
        assert_eq!(center_height.get(edge_x, y), right_height.get(0, y));
    }

    for x in 0..negative_z_height.width() {
        assert_eq!(negative_z_height.get(x, edge_y), center_height.get(x, 0));
        assert_eq!(center_height.get(x, edge_y), positive_z_height.get(x, 0));
    }
}

fn assert_right_edge_matches_left_edge(
    left: &TerrainMesh,
    right: &TerrainMesh,
    width: usize,
    height: usize,
) {
    const POSITION_EPSILON: f64 = 1.0e-5;
    let left_x = width - 1;

    for y in 0..height {
        let left_position = reconstructed_world_position(left, y * width + left_x);
        let right_position = reconstructed_world_position(right, y * width);

        assert!(
            vector_distance(left_position, right_position) <= POSITION_EPSILON,
            "edge vertex {y} differed: {left_position:?} vs {right_position:?}"
        );
    }
}

fn assert_positive_z_edge_matches_negative_z_edge(
    negative_z: &TerrainMesh,
    positive_z: &TerrainMesh,
    width: usize,
    height: usize,
) {
    const POSITION_EPSILON: f64 = 1.0e-5;
    let negative_z_y = height - 1;

    for x in 0..width {
        let negative_z_position =
            reconstructed_world_position(negative_z, negative_z_y * width + x);
        let positive_z_position = reconstructed_world_position(positive_z, x);

        assert!(
            vector_distance(negative_z_position, positive_z_position) <= POSITION_EPSILON,
            "edge vertex {x} differed: {negative_z_position:?} vs {positive_z_position:?}"
        );
    }
}

fn reconstructed_world_position(mesh: &TerrainMesh, index: usize) -> Vector3F64 {
    let origin = mesh.origin();
    let local = mesh.positions()[index];

    Vector3F64::new(
        origin.x + f64::from(local.x),
        origin.y + f64::from(local.y),
        origin.z + f64::from(local.z),
    )
}

fn vector_distance(lhs: Vector3F64, rhs: Vector3F64) -> f64 {
    let dx = lhs.x - rhs.x;
    let dy = lhs.y - rhs.y;
    let dz = lhs.z - rhs.z;

    (dx * dx + dy * dy + dz * dz).sqrt()
}
