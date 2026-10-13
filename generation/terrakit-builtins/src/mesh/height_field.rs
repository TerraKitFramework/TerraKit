//! Heightfield-to-mesh stage implementation.

use terrakit_algorithms::{
    AlgorithmError,
    mesh::{generate_grid_texcoords, generate_grid_triangle_indices, generate_smooth_normals},
};
use terrakit_core::{CoreError, HeightField, MeshIndex, TerrainMesh, TerrainResource, Vector3F32};
use terrakit_pipeline::{
    ResourceAccess, ResourceKey, ResourceKind, ResourceProduct, ResourceRequirement, ResourceSet,
    StageContext, StageError, StageId, TerrainStage,
};

const STAGE_NAME: &str = "height_field_mesh";

/// Terrain stage that converts a canonical height field into a canonical mesh.
///
/// The generated mesh keeps a high-precision world origin and stores vertex
/// positions as `f32` values relative to that origin. Vertices are emitted in
/// X-contiguous grid order (`index = y * width + x`), matching the reusable
/// grid-index and UV helpers. Mesh position continuity across neighbouring
/// regions is guaranteed by reconstructing world positions as
/// `mesh.origin() + local_position`. Cross-region smooth-normal continuity is
/// deferred because independently generated regions do not yet exchange border
/// samples or reconcile edge normals.
#[derive(Debug, Clone)]
pub struct HeightFieldMeshStage {
    stage_id: StageId,
    input: ResourceKey,
    output: ResourceKey,
    generate_normals: bool,
    generate_texcoords: bool,
    replace_output: bool,
    requirements: [ResourceRequirement; 1],
    products: [ResourceProduct; 1],
}

impl HeightFieldMeshStage {
    /// Creates a heightfield-to-mesh stage with normals and texcoords enabled.
    ///
    /// The stage creates `output` as a `Mesh` resource and reads `input` as a
    /// `HeightField`. Input and output keys must be distinct at execution time
    /// so the source heightfield remains available after meshing.
    pub fn new(stage_id: StageId, input: ResourceKey, output: ResourceKey) -> Self {
        Self {
            stage_id,
            input,
            output,
            generate_normals: true,
            generate_texcoords: true,
            replace_output: false,
            requirements: [ResourceRequirement::required(
                input,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )],
            products: [ResourceProduct::create(output, ResourceKind::Mesh)],
        }
    }

    /// Enables or disables smooth normal generation.
    pub fn with_normals(mut self, enabled: bool) -> Self {
        self.generate_normals = enabled;
        self
    }

    /// Enables or disables regular-grid texture-coordinate generation.
    pub fn with_texcoords(mut self, enabled: bool) -> Self {
        self.generate_texcoords = enabled;
        self
    }

    /// Enables or disables replacing an existing mesh at the output key.
    pub fn with_output_replacement(mut self, enabled: bool) -> Self {
        self.replace_output = enabled;
        self.products = [if enabled {
            ResourceProduct::replace(self.output, ResourceKind::Mesh)
        } else {
            ResourceProduct::create(self.output, ResourceKind::Mesh)
        }];
        self
    }

    /// Returns the heightfield input key.
    pub const fn input(&self) -> ResourceKey {
        self.input
    }

    /// Returns the mesh output key.
    pub const fn output(&self) -> ResourceKey {
        self.output
    }

    /// Returns whether smooth normals will be generated.
    pub const fn normals_enabled(&self) -> bool {
        self.generate_normals
    }

    /// Returns whether regular-grid texture coordinates will be generated.
    pub const fn texcoords_enabled(&self) -> bool {
        self.generate_texcoords
    }

    /// Returns whether an existing output mesh may be replaced.
    pub const fn output_replacement_enabled(&self) -> bool {
        self.replace_output
    }
}

impl TerrainStage for HeightFieldMeshStage {
    fn id(&self) -> StageId {
        self.stage_id
    }

    fn name(&self) -> &str {
        STAGE_NAME
    }

    fn requirements(&self) -> &[ResourceRequirement] {
        &self.requirements
    }

    fn products(&self) -> &[ResourceProduct] {
        &self.products
    }

    fn execute(
        &mut self,
        context: &StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), StageError> {
        let _region = context.region2().map_err(|_| {
            StageError::new("heightfield mesh stage requires a 2D generation region")
        })?;

        if self.input == self.output {
            return Err(StageError::new(
                "heightfield mesh input and output keys must be distinct",
            ));
        }

        let Some(height_field) = resources.height_field(self.input) else {
            return Err(StageError::new(format!(
                "heightfield mesh input {:?} is missing or is not a height field",
                self.input
            )));
        };

        if height_field.width() < 2 || height_field.height() < 2 {
            return Err(StageError::new(
                "heightfield mesh generation requires at least 2x2 samples",
            ));
        }

        if !self.replace_output && resources.contains(self.output) {
            return Err(StageError::new(format!(
                "cannot create terrain mesh at {:?}: resource already exists",
                self.output
            )));
        }

        let mesh = build_mesh(height_field, self.generate_normals, self.generate_texcoords)?;
        resources.insert(self.output, TerrainResource::Mesh(mesh));

        Ok(())
    }
}

fn build_mesh(
    height_field: &HeightField,
    generate_normals: bool,
    generate_texcoords: bool,
) -> Result<TerrainMesh, StageError> {
    let extent = height_field.extent();
    let width = extent.width();
    let height = extent.height();
    let origin = height_field.transform().origin();
    let positions = generate_positions(height_field)?;
    let indices = generate_grid_triangle_indices(width, height).map_err(|error| {
        algorithm_stage_error("failed to generate heightfield mesh indices", error)
    })?;
    let normals = if generate_normals {
        Some(
            generate_smooth_normals(&positions, &indices).map_err(|error| {
                algorithm_stage_error("failed to generate heightfield mesh normals", error)
            })?,
        )
    } else {
        None
    };
    let texcoords = if generate_texcoords {
        Some(generate_grid_texcoords(width, height).map_err(|error| {
            algorithm_stage_error("failed to generate heightfield mesh texcoords", error)
        })?)
    } else {
        None
    };

    TerrainMesh::new(origin, positions, indices, normals, texcoords)
        .map_err(|error| core_stage_error("failed to construct canonical terrain mesh", error))
}

fn generate_positions(height_field: &HeightField) -> Result<Vec<Vector3F32>, StageError> {
    let extent = height_field.extent();
    let vertex_count = checked_vertex_count(extent.width(), extent.height())?;
    let origin = height_field.transform().origin();
    let mut positions = Vec::new();
    positions
        .try_reserve_exact(vertex_count)
        .map_err(|_| StageError::new("failed to allocate heightfield mesh positions"))?;

    for y in 0..height_field.height() {
        for x in 0..height_field.width() {
            let Some(world_position) = height_field.surface_position(x, y) else {
                return Err(StageError::new(format!(
                    "missing heightfield surface position at ({x}, {y})"
                )));
            };
            let local_x = checked_f64_to_f32(world_position.x - origin.x, "x").map_err(|error| {
                StageError::new(format!(
                    "heightfield vertex at ({x}, {y}) could not be represented as local f32 position: {error}"
                ))
            })?;
            let local_y = checked_f64_to_f32(world_position.y - origin.y, "y").map_err(|error| {
                StageError::new(format!(
                    "heightfield vertex at ({x}, {y}) could not be represented as local f32 position: {error}"
                ))
            })?;
            let local_z = checked_f64_to_f32(world_position.z - origin.z, "z").map_err(|error| {
                StageError::new(format!(
                    "heightfield vertex at ({x}, {y}) could not be represented as local f32 position: {error}"
                ))
            })?;

            positions.push(Vector3F32::new(local_x, local_y, local_z));
        }
    }

    Ok(positions)
}

fn checked_vertex_count(width: u32, height: u32) -> Result<usize, StageError> {
    let vertex_count = u64::from(width)
        .checked_mul(u64::from(height))
        .ok_or_else(|| StageError::new("heightfield mesh vertex count overflowed"))?;

    if vertex_count > u64::from(MeshIndex::MAX) {
        return Err(StageError::new(
            "heightfield mesh vertex count exceeds mesh index capacity",
        ));
    }

    usize::try_from(vertex_count)
        .map_err(|_| StageError::new("heightfield mesh vertex count exceeds address space"))
}

fn checked_f64_to_f32(value: f64, component: &'static str) -> Result<f32, StageError> {
    if !value.is_finite() {
        return Err(StageError::new(format!(
            "local {component} component is not finite"
        )));
    }

    if value < f64::from(f32::MIN) || value > f64::from(f32::MAX) {
        return Err(StageError::new(format!(
            "local {component} component is outside finite f32 range"
        )));
    }

    let converted = value as f32;
    if !converted.is_finite() {
        return Err(StageError::new(format!(
            "local {component} component converted to a non-finite f32"
        )));
    }

    Ok(converted)
}

fn algorithm_stage_error(operation: &str, error: AlgorithmError) -> StageError {
    StageError::new(format!("{operation}: {error}"))
}

fn core_stage_error(operation: &str, error: CoreError) -> StageError {
    StageError::new(format!("{operation}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use terrakit_core::{
        Extent2, Extent3, GenerationRegion, GenerationSeed, GridTransform2, HeightField, LodLevel,
        RegionCoord2, RegionCoord3, RegionLayout2, RegionLayout3, RegionRequest2, RegionRequest3,
        SamplingDomain, TerrainResource, Vector2F64, Vector3F64,
    };
    use terrakit_pipeline::{ResourceKind, TerrainPipeline};

    fn layout2(width: u32, height: u32, spacing_x: f64, spacing_y: f64) -> RegionLayout2 {
        RegionLayout2::new(
            Extent2::try_new(width, height).unwrap(),
            Vector2F64::new(spacing_x, spacing_y),
        )
        .unwrap()
    }

    fn context2(layout: RegionLayout2, coordinate: RegionCoord2) -> StageContext {
        let region = GenerationRegion::Region2(
            layout
                .resolve(RegionRequest2::new(coordinate, LodLevel::HIGHEST))
                .unwrap(),
        );

        StageContext::new(GenerationSeed::new(123), region)
    }

    fn context3() -> StageContext {
        let layout = RegionLayout3::new(
            Extent3::try_new(2, 2, 2).unwrap(),
            Vector3F64::new(1.0, 1.0, 1.0),
        )
        .unwrap();
        let region = GenerationRegion::Region3(
            layout
                .resolve(RegionRequest3::new(RegionCoord3::ZERO, LodLevel::HIGHEST))
                .unwrap(),
        );

        StageContext::new(GenerationSeed::new(123), region)
    }

    fn height_field_for_context(context: &StageContext, value: f32) -> HeightField {
        let region = context.region2().unwrap();

        HeightField::filled(
            region.point_sample_extent(),
            value,
            region.transform(),
            SamplingDomain::Points,
            Vector3F64::Y,
        )
        .unwrap()
    }

    fn custom_height_field(
        width: u32,
        height: u32,
        transform: GridTransform2,
        height_axis: Vector3F64,
    ) -> HeightField {
        HeightField::filled(
            Extent2::try_new(width, height).unwrap(),
            0.0,
            transform,
            SamplingDomain::Points,
            height_axis,
        )
        .unwrap()
    }

    fn execute_stage(
        mut stage: HeightFieldMeshStage,
        context: &StageContext,
        height_field: HeightField,
    ) -> ResourceSet {
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field),
        );

        stage.execute(context, &mut resources).unwrap();

        resources
    }

    #[test]
    fn declares_heightfield_requirement_and_mesh_product() {
        let stage = HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH);

        assert_eq!(stage.id(), StageId(3));
        assert_eq!(stage.name(), "height_field_mesh");
        assert_eq!(stage.input(), ResourceKey::HEIGHT);
        assert_eq!(stage.output(), ResourceKey::MESH);
        assert!(stage.normals_enabled());
        assert!(stage.texcoords_enabled());
        assert!(!stage.output_replacement_enabled());
        assert_eq!(
            stage.requirements(),
            &[ResourceRequirement::required(
                ResourceKey::HEIGHT,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )]
        );
        assert_eq!(
            stage.products(),
            &[ResourceProduct::create(
                ResourceKey::MESH,
                ResourceKind::Mesh
            )]
        );
    }

    #[test]
    fn output_replacement_changes_product_declaration() {
        let stage = HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH)
            .with_output_replacement(true);

        assert!(stage.output_replacement_enabled());
        assert_eq!(
            stage.products(),
            &[ResourceProduct::replace(
                ResourceKey::MESH,
                ResourceKind::Mesh
            )]
        );
    }

    #[test]
    fn does_not_mutate_heightfield() {
        let context = context2(layout2(2, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let height_field = height_field_for_context(&context, 3.0);
        let original = height_field.clone();
        let resources = execute_stage(
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH),
            &context,
            height_field,
        );

        assert_eq!(resources.height_field(ResourceKey::HEIGHT), Some(&original));
        assert!(resources.mesh(ResourceKey::MESH).is_some());
    }

    #[test]
    fn direct_execution_reports_missing_input() {
        let context = context2(layout2(2, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let mut resources = ResourceSet::new();
        let mut stage =
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH);

        let error = stage.execute(&context, &mut resources).unwrap_err();

        assert!(error.message().contains("is missing"));
    }

    #[test]
    fn rejects_fields_smaller_than_two_by_two() {
        let context = context2(layout2(2, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let height_field = custom_height_field(1, 2, GridTransform2::identity_xz(), Vector3F64::Y);
        let mut stage =
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field),
        );

        let error = stage.execute(&context, &mut resources).unwrap_err();

        assert!(error.message().contains("at least 2x2 samples"));
        assert!(resources.mesh(ResourceKey::MESH).is_none());
    }

    #[test]
    fn produces_expected_counts_and_valid_indices() {
        let context = context2(layout2(2, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let resources = execute_stage(
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH),
            &context,
            height_field_for_context(&context, 0.0),
        );
        let mesh = resources.mesh(ResourceKey::MESH).unwrap();

        assert_eq!(mesh.vertex_count(), 9);
        assert_eq!(mesh.index_count(), 24);
        assert_eq!(mesh.triangle_count(), 8);
        assert!(mesh.indices().iter().all(|index| *index < 9));
    }

    #[test]
    fn optional_normals_and_texcoords_follow_configuration() {
        let context = context2(layout2(1, 1, 1.0, 1.0), RegionCoord2::ZERO);
        let full = execute_stage(
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH),
            &context,
            height_field_for_context(&context, 0.0),
        );
        let sparse = execute_stage(
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH)
                .with_normals(false)
                .with_texcoords(false),
            &context,
            height_field_for_context(&context, 0.0),
        );

        assert!(full.mesh(ResourceKey::MESH).unwrap().normals().is_some());
        assert!(full.mesh(ResourceKey::MESH).unwrap().texcoords().is_some());
        assert!(sparse.mesh(ResourceKey::MESH).unwrap().normals().is_none());
        assert!(
            sparse
                .mesh(ResourceKey::MESH)
                .unwrap()
                .texcoords()
                .is_none()
        );
    }

    #[test]
    fn preserves_transform_origin_and_stores_local_positions() {
        let context = context2(layout2(2, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let transform = GridTransform2::from_axes(
            Vector3F64::new(100.0, 5.0, -200.0),
            Vector3F64::new(2.0, 0.0, 0.0),
            Vector3F64::new(0.0, 0.0, 3.0),
        )
        .unwrap();
        let mut height_field = custom_height_field(2, 2, transform, Vector3F64::Y);
        height_field.try_set(1, 1, 4.0).unwrap();
        let resources = execute_stage(
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH),
            &context,
            height_field,
        );
        let mesh = resources.mesh(ResourceKey::MESH).unwrap();

        assert_eq!(mesh.origin(), transform.origin());
        assert_eq!(mesh.positions()[0], Vector3F32::ZERO);
        assert_eq!(mesh.positions()[3], Vector3F32::new(2.0, 4.0, 3.0));
    }

    #[test]
    fn handles_rotated_valid_transforms() {
        let context = context2(layout2(2, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let transform = GridTransform2::from_axes(
            Vector3F64::new(10.0, 0.0, 20.0),
            Vector3F64::new(0.0, 0.0, 2.0),
            Vector3F64::new(-3.0, 0.0, 0.0),
        )
        .unwrap();
        let mut height_field = custom_height_field(2, 2, transform, Vector3F64::Y);
        height_field.try_set(1, 1, 1.5).unwrap();
        let resources = execute_stage(
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH),
            &context,
            height_field,
        );
        let mesh = resources.mesh(ResourceKey::MESH).unwrap();

        assert_eq!(mesh.positions()[3], Vector3F32::new(-3.0, 1.5, 2.0));
    }

    #[test]
    fn deterministic_output() {
        let context = context2(layout2(3, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let first = execute_stage(
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH),
            &context,
            height_field_for_context(&context, 1.0),
        );
        let second = execute_stage(
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH),
            &context,
            height_field_for_context(&context, 1.0),
        );

        assert_eq!(
            first.mesh(ResourceKey::MESH),
            second.mesh(ResourceKey::MESH)
        );
    }

    #[test]
    fn rejects_3d_generation_context() {
        let mut stage =
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(HeightField::from_dimensions(2, 2).unwrap()),
        );

        let error = stage.execute(&context3(), &mut resources).unwrap_err();

        assert!(error.message().contains("requires a 2D generation region"));
    }

    #[test]
    fn rejects_identical_input_and_output_keys() {
        let context = context2(layout2(2, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let mut stage =
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::HEIGHT)
                .with_output_replacement(true);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(height_field_for_context(&context, 0.0)),
        );

        let error = stage.execute(&context, &mut resources).unwrap_err();

        assert!(error.message().contains("must be distinct"));
    }

    #[test]
    fn converts_position_conversion_errors_to_stage_errors() {
        let context = context2(layout2(2, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let transform = GridTransform2::from_axes(
            Vector3F64::ZERO,
            Vector3F64::new(f64::from(f32::MAX) * 2.0, 0.0, 0.0),
            Vector3F64::Z,
        )
        .unwrap();
        let mut stage =
            HeightFieldMeshStage::new(StageId(3), ResourceKey::HEIGHT, ResourceKey::MESH);
        let mut resources = ResourceSet::new();
        resources.insert(
            ResourceKey::HEIGHT,
            TerrainResource::HeightField(custom_height_field(2, 2, transform, Vector3F64::Y)),
        );

        let error = stage.execute(&context, &mut resources).unwrap_err();

        assert!(error.message().contains("local f32 position"));
        assert!(resources.mesh(ResourceKey::MESH).is_none());
    }

    #[test]
    fn runs_after_flat_height_stage_in_pipeline() {
        let context = context2(layout2(2, 2, 1.0, 1.0), RegionCoord2::ZERO);
        let mut pipeline = TerrainPipeline::new();
        pipeline.add_stage(
            crate::FlatHeightStage::standard(StageId(1), ResourceKey::HEIGHT, 2.0).unwrap(),
        );
        pipeline.add_stage(HeightFieldMeshStage::new(
            StageId(3),
            ResourceKey::HEIGHT,
            ResourceKey::MESH,
        ));
        let mut resources = ResourceSet::new();

        pipeline.execute(&context, &mut resources).unwrap();

        assert_eq!(
            resources.kind(ResourceKey::HEIGHT),
            Some(ResourceKind::HeightField)
        );
        assert_eq!(resources.kind(ResourceKey::MESH), Some(ResourceKind::Mesh));
    }
}
