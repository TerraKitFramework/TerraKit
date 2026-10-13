//! Serial terrain pipeline execution and contract validation.

use crate::{
    PipelineError, ResourceProduct, ResourceRequirement, ResourceSet, StageId, TerrainStage,
};

/// Serial terrain pipeline that executes stages in insertion order.
#[derive(Default)]
pub struct TerrainPipeline {
    stages: Vec<Box<dyn TerrainStage>>,
}

impl TerrainPipeline {
    /// Creates an empty terrain pipeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of stages in the pipeline.
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Returns true when the pipeline contains no stages.
    pub fn is_empty(&self) -> bool {
        self.stages.is_empty()
    }

    /// Appends a terrain stage to the pipeline.
    pub fn add_stage<T>(&mut self, stage: T)
    where
        T: TerrainStage + 'static,
    {
        self.add_boxed_stage(Box::new(stage));
    }

    /// Appends an already boxed terrain stage to the pipeline.
    pub fn add_boxed_stage(&mut self, stage: Box<dyn TerrainStage>) {
        self.stages.push(stage);
    }

    /// Executes each stage serially against `resources`.
    ///
    /// Execution is not transactional. When a stage fails, resources may
    /// contain modifications made by that stage before it returned an error.
    pub fn execute(
        &mut self,
        context: &crate::StageContext,
        resources: &mut ResourceSet,
    ) -> Result<(), PipelineError> {
        for stage in &mut self.stages {
            let stage_id = stage.id();
            let stage_name = stage.name().to_owned();

            validate_requirements(stage_id, &stage_name, stage.requirements(), resources)?;
            validate_product_collisions(stage_id, &stage_name, stage.products(), resources)?;

            stage
                .execute(context, resources)
                .map_err(|source| PipelineError::StageFailed {
                    stage_id,
                    stage_name: stage_name.clone(),
                    source,
                })?;

            validate_outputs(stage_id, &stage_name, stage.products(), resources)?;
        }

        Ok(())
    }
}

fn validate_requirements(
    stage_id: StageId,
    stage_name: &str,
    requirements: &[ResourceRequirement],
    resources: &ResourceSet,
) -> Result<(), PipelineError> {
    for requirement in requirements {
        match resources.kind(requirement.key) {
            Some(actual) if actual == requirement.kind => {}
            Some(actual) => {
                return Err(PipelineError::WrongResourceKind {
                    stage_id,
                    stage_name: stage_name.to_owned(),
                    key: requirement.key,
                    expected: requirement.kind,
                    actual,
                });
            }
            None if requirement.optional => {}
            None => {
                return Err(PipelineError::MissingResource {
                    stage_id,
                    stage_name: stage_name.to_owned(),
                    key: requirement.key,
                });
            }
        }
    }

    Ok(())
}

fn validate_product_collisions(
    stage_id: StageId,
    stage_name: &str,
    products: &[ResourceProduct],
    resources: &ResourceSet,
) -> Result<(), PipelineError> {
    for product in products {
        if !product.replace && resources.contains(product.key) {
            return Err(PipelineError::ResourceAlreadyExists {
                stage_id,
                stage_name: stage_name.to_owned(),
                key: product.key,
            });
        }
    }

    Ok(())
}

fn validate_outputs(
    stage_id: StageId,
    stage_name: &str,
    products: &[ResourceProduct],
    resources: &ResourceSet,
) -> Result<(), PipelineError> {
    for product in products {
        match resources.kind(product.key) {
            Some(actual) if actual == product.kind => {}
            Some(actual) => {
                return Err(PipelineError::WrongOutputKind {
                    stage_id,
                    stage_name: stage_name.to_owned(),
                    key: product.key,
                    expected: product.kind,
                    actual,
                });
            }
            None => {
                return Err(PipelineError::MissingOutput {
                    stage_id,
                    stage_name: stage_name.to_owned(),
                    key: product.key,
                });
            }
        }
    }

    Ok(())
}
