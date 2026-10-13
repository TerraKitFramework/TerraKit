//! Height-field amplification stage implementation.

use terrakit_core::TerrainResource;
use terrakit_pipeline::{
    ResourceAccess, ResourceKey, ResourceKind, ResourceProduct, ResourceRequirement, ResourceSet,
    StageContext, StageError, StageId, TerrainStage,
};

const STAGE_NAME: &str = "amplify_height";

/// Amplification mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeightAmplifyMode {
    /// Add to noise
    #[default]
    Add,
    /// Mulitply noise
    Multiply,
}

impl HeightAmplifyMode {
    fn combine(self, existing: f32, amplify_value: f32) -> f32 {
        match self {
            Self::Add => existing + amplify_value,
            Self::Multiply => existing * amplify_value,
        }
    }
}

/// Terrain stage that amplifies existing height values by a user-defined amplification value.
#[derive(Debug, Clone)]
pub struct AmplifyHeightStage {
    stage_id: StageId,
    input: ResourceKey,
    output: ResourceKey,
    mode: HeightAmplifyMode,
    threshold_min: f32,
    threshold_max: f32,
    amplify_value: f32,
    amplify_limit: f32,
    requirements: [ResourceRequirement; 1],
    products: [ResourceProduct; 1],
}

impl AmplifyHeightStage {
    /// Create amplification stage
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        stage_id: StageId,
        input: ResourceKey,
        output: ResourceKey,
        mode: HeightAmplifyMode,
        threshold_min: f32,
        threshold_max: f32,
        amplify_value: f32,
        amplify_limit: f32,
    ) -> Result<Self, StageError> {
        if !threshold_min.is_finite() && !threshold_max.is_finite() {
            return Err(StageError::new("amplify threshold must be finite"));
        }

        if !amplify_value.is_finite() {
            return Err(StageError::new("amplify value must be finite"));
        }

        if !amplify_limit.is_finite() {
            return Err(StageError::new("amplify limit must be finite"));
        }

        Ok(Self {
            stage_id,
            input,
            output,
            mode,
            threshold_min,
            threshold_max,
            amplify_value,
            amplify_limit,
            requirements: [ResourceRequirement::required(
                input,
                ResourceKind::HeightField,
                ResourceAccess::Read,
            )],
            products: [ResourceProduct::create(output, ResourceKind::HeightField)],
        })
    }

    /// Returns the height-field key this stage reads.
    pub const fn input(&self) -> ResourceKey {
        self.input
    }

    /// Returns the height-field key this stage creates.
    pub const fn output(&self) -> ResourceKey {
        self.output
    }

    /// Returns the configured amplification mode.
    pub const fn mode(&self) -> HeightAmplifyMode {
        self.mode
    }

    /// Returns the starting height threshold
    pub const fn threshold_min(&self) -> f32 {
        self.threshold_min
    }

    /// Returns the threshold direction boolean
    pub const fn threshold_max(&self) -> f32 {
        self.threshold_max
    }

    /// Returns the given amplification value
    pub const fn amplify_value(&self) -> f32 {
        self.amplify_value
    }

    /// Returns the given amplification limit
    pub const fn amplify_limit(&self) -> f32 {
        self.amplify_limit
    }
}

impl TerrainStage for AmplifyHeightStage {
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
        let _region = context.region().as_region2().ok_or_else(|| {
            StageError::new("amplify height stage requires a 2D generation region")
        })?;

        if self.input == self.output {
            return Err(StageError::new(
                "height amplify input and output keys must be distinct",
            ));
        }

        if resources.contains(self.output) {
            return Err(StageError::new(format!(
                "cannot create height amplify output at {:?}: resource already exists",
                self.output
            )));
        }

        let Some(source_height_field) = resources.height_field(self.input) else {
            return Err(StageError::new(format!(
                "height amplify input {:?} is missing or is not a height field",
                self.input
            )));
        };
        let mut output_height_field = source_height_field.clone();

        for y in 0..output_height_field.height() {
            for x in 0..output_height_field.width() {
                let Some(existing_height) = output_height_field.get(x, y) else {
                    return Err(StageError::new(format!(
                        "missing height sample at ({x}, {y})"
                    )));
                };

                if existing_height < self.threshold_min || existing_height > self.threshold_max {
                    continue;
                }

                let mut next = self.mode.combine(existing_height, self.amplify_value);

                if !next.is_finite() {
                    return Err(StageError::new(format!(
                        "height amplify produced a non-finite sample at ({x}, {y})"
                    )));
                }

                if next > existing_height
                    && existing_height < self.amplify_limit
                    && next > self.amplify_limit
                {
                    next = self.amplify_limit;
                }

                if next < existing_height
                    && existing_height > self.amplify_limit
                    && next < self.amplify_limit
                {
                    next = self.amplify_limit;
                }

                output_height_field.try_set(x, y, next).map_err(|error| {
                    StageError::new(format!(
                        "failed to write height sample at ({x}, {y}): {error}"
                    ))
                })?;
            }
        }

        resources.insert(
            self.output,
            TerrainResource::HeightField(output_height_field),
        );

        Ok(())
    }
}
