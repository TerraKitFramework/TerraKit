//! Assembly of schema-validated stage construction requests into pipelines.

use std::collections::{HashMap, HashSet};

use crate::{
    ResourceAccess, ResourceKey, ResourceKind, ResourceProduct, ResourceRequirement, StageId,
    TerrainPipeline, TerrainStage,
};

use super::{
    BindingError, ParameterSet, PipelineAssemblyError, ResolvedParameterSet, StageBindings,
    StageBuildError, StageRegistry, StageSchema, StageSchemaVersion, StageTypeId,
    ValidatedStageBindings,
};

/// Discoverable stage type metadata plus factory for executable stages.
pub trait StageDefinition: Send + Sync {
    /// Returns discoverable metadata for this stage type.
    fn schema(&self) -> &StageSchema;

    /// Constructs one executable stage instance from resolved values.
    fn build(
        &self,
        stage_id: StageId,
        parameters: &ResolvedParameterSet,
        bindings: &ValidatedStageBindings,
    ) -> Result<Box<dyn TerrainStage>, StageBuildError>;
}

/// Ordered request to construct one executable terrain stage instance.
///
/// This is not a visual node. It contains only the stage instance ID, stage
/// type, requested schema version, typed parameter values, and resolved
/// port-resource bindings needed to construct a concrete [`TerrainStage`].
/// Saved interface nodes must submit the schema version stored with that node;
/// callers should not silently substitute the currently registered version.
#[derive(Debug, Clone)]
pub struct StageConstruction {
    stage_id: StageId,
    stage_type: StageTypeId,
    schema_version: StageSchemaVersion,
    parameters: ParameterSet,
    bindings: StageBindings,
}

impl StageConstruction {
    /// Creates a stage construction request with no parameters or bindings.
    pub fn new(
        stage_id: StageId,
        stage_type: StageTypeId,
        schema_version: StageSchemaVersion,
    ) -> Self {
        Self {
            stage_id,
            stage_type,
            schema_version,
            parameters: ParameterSet::new(),
            bindings: StageBindings::new(),
        }
    }

    /// Creates a fresh construction request from the schema's exact type and version.
    pub fn from_schema(stage_id: StageId, schema: &StageSchema) -> Self {
        Self::new(stage_id, schema.type_id().clone(), schema.version())
    }

    /// Returns the executable stage instance ID.
    pub const fn stage_id(&self) -> StageId {
        self.stage_id
    }

    /// Returns the requested stage type ID.
    pub fn stage_type(&self) -> &StageTypeId {
        &self.stage_type
    }

    /// Returns the requested stage schema version.
    pub const fn schema_version(&self) -> StageSchemaVersion {
        self.schema_version
    }

    /// Returns the supplied parameter set.
    pub fn parameters(&self) -> &ParameterSet {
        &self.parameters
    }

    /// Returns the supplied port bindings.
    pub fn bindings(&self) -> &StageBindings {
        &self.bindings
    }

    /// Returns the request with a replacement parameter set.
    pub fn with_parameters(mut self, parameters: ParameterSet) -> Self {
        self.parameters = parameters;
        self
    }

    /// Returns the request with replacement port bindings.
    pub fn with_bindings(mut self, bindings: StageBindings) -> Self {
        self.bindings = bindings;
        self
    }
}

/// Assembles caller-ordered stage construction requests into a terrain pipeline.
///
/// The assembler validates parameters, port-resource bindings, resource
/// availability, resource kinds, unique producers, and the executable stage
/// contract. It preserves the caller's order and never performs topological
/// sorting or graph traversal.
pub struct TerrainPipelineAssembler<'a> {
    registry: &'a StageRegistry,
    pipeline: TerrainPipeline,
    produced_resources: HashMap<ResourceKey, ResourceKind>,
    stage_ids: HashSet<StageId>,
}

impl<'a> TerrainPipelineAssembler<'a> {
    /// Creates an empty assembler backed by a stage registry.
    pub fn new(registry: &'a StageRegistry) -> Self {
        Self {
            registry,
            pipeline: TerrainPipeline::new(),
            produced_resources: HashMap::new(),
            stage_ids: HashSet::new(),
        }
    }

    /// Validates, builds, and appends one stage in caller-supplied order.
    pub fn add_stage(
        &mut self,
        construction: StageConstruction,
    ) -> Result<(), PipelineAssemblyError> {
        let StageConstruction {
            stage_id,
            stage_type,
            schema_version,
            parameters,
            bindings,
        } = construction;

        if self.stage_ids.contains(&stage_id) {
            return Err(PipelineAssemblyError::DuplicateStageId { stage_id });
        }

        let definition = self.registry.definition(&stage_type).ok_or_else(|| {
            PipelineAssemblyError::UnknownStageType {
                stage_type: stage_type.clone(),
            }
        })?;
        let schema = definition.schema();
        if schema_version != schema.version() {
            return Err(PipelineAssemblyError::SchemaVersionMismatch {
                stage_id,
                stage_type,
                requested: schema_version,
                available: schema.version(),
            });
        }

        let resolved_parameters = schema
            .resolve_parameters(&parameters)
            .map_err(|source| PipelineAssemblyError::Parameter { stage_id, source })?;
        let validated_bindings =
            ValidatedStageBindings::validate(schema, &bindings).map_err(|source| match source {
                BindingError::InputOutputAlias { resource } => {
                    PipelineAssemblyError::InputOutputAlias {
                        stage_id,
                        key: resource,
                    }
                }
                other => PipelineAssemblyError::Binding {
                    stage_id,
                    source: other,
                },
            })?;

        self.validate_input_availability(stage_id, schema, &validated_bindings)?;
        self.validate_output_uniqueness(stage_id, schema, &validated_bindings)?;

        let stage = definition
            .build(stage_id, &resolved_parameters, &validated_bindings)
            .map_err(|source| PipelineAssemblyError::StageBuild {
                stage_id,
                stage_type: stage_type.clone(),
                source,
            })?;

        verify_stage_contract(stage.as_ref(), stage_id, schema, &validated_bindings).map_err(
            |message| PipelineAssemblyError::StageContractMismatch { stage_id, message },
        )?;

        for (port, key) in validated_bindings.outputs() {
            let Some(output) = schema.output(port) else {
                return Err(PipelineAssemblyError::StageContractMismatch {
                    stage_id,
                    message: format!("validated output port '{port}' is not in schema"),
                });
            };
            self.produced_resources.insert(*key, output.resource_kind());
        }
        self.stage_ids.insert(stage_id);
        self.pipeline.add_boxed_stage(stage);

        Ok(())
    }

    /// Consumes the assembler and returns the executable terrain pipeline.
    pub fn finish(self) -> TerrainPipeline {
        self.pipeline
    }

    fn validate_input_availability(
        &self,
        stage_id: StageId,
        schema: &StageSchema,
        bindings: &ValidatedStageBindings,
    ) -> Result<(), PipelineAssemblyError> {
        for (port, key) in bindings.inputs() {
            let Some(input) = schema.input(port) else {
                return Err(PipelineAssemblyError::StageContractMismatch {
                    stage_id,
                    message: format!("validated input port '{port}' is not in schema"),
                });
            };

            match self.produced_resources.get(key) {
                Some(actual) if *actual == input.resource_kind() => {}
                Some(actual) => {
                    return Err(PipelineAssemblyError::WrongInputResourceKind {
                        stage_id,
                        port: port.clone(),
                        key: *key,
                        expected: input.resource_kind(),
                        actual: *actual,
                    });
                }
                None => {
                    return Err(PipelineAssemblyError::MissingInputResource {
                        stage_id,
                        port: port.clone(),
                        key: *key,
                    });
                }
            }
        }

        Ok(())
    }

    fn validate_output_uniqueness(
        &self,
        stage_id: StageId,
        schema: &StageSchema,
        bindings: &ValidatedStageBindings,
    ) -> Result<(), PipelineAssemblyError> {
        for (port, key) in bindings.outputs() {
            if self.produced_resources.contains_key(key) {
                return Err(PipelineAssemblyError::OutputResourceAlreadyAssigned {
                    stage_id,
                    port: port.clone(),
                    key: *key,
                });
            }

            if bindings.inputs().any(|(_, input_key)| input_key == key) {
                return Err(PipelineAssemblyError::InputOutputAlias {
                    stage_id,
                    key: *key,
                });
            }

            if schema.output(port).is_none() {
                return Err(PipelineAssemblyError::StageContractMismatch {
                    stage_id,
                    message: format!("validated output port '{port}' is not in schema"),
                });
            }
        }

        Ok(())
    }
}

fn verify_stage_contract(
    stage: &dyn TerrainStage,
    stage_id: StageId,
    schema: &StageSchema,
    bindings: &ValidatedStageBindings,
) -> Result<(), String> {
    if stage.id() != stage_id {
        return Err(format!(
            "factory returned stage id {:?}, expected {:?}",
            stage.id(),
            stage_id
        ));
    }

    verify_requirements(stage.requirements(), schema, bindings)?;
    verify_products(stage.products(), schema, bindings)
}

fn verify_requirements(
    requirements: &[ResourceRequirement],
    schema: &StageSchema,
    bindings: &ValidatedStageBindings,
) -> Result<(), String> {
    let mut remaining = requirements.to_vec();

    for (port, key) in bindings.inputs() {
        let Some(input) = schema.input(port) else {
            return Err(format!("bound input port '{port}' is missing from schema"));
        };
        let expected =
            ResourceRequirement::required(*key, input.resource_kind(), ResourceAccess::Read);
        let Some(index) = remaining
            .iter()
            .position(|requirement| *requirement == expected)
        else {
            return Err(format!(
                "missing read-only requirement for input port '{port}' bound to {key:?}"
            ));
        };

        remaining.remove(index);
    }

    if let Some(requirement) = remaining.first() {
        return Err(format!(
            "undeclared requirement for resource {:?} ({:?}, {:?})",
            requirement.key, requirement.kind, requirement.access
        ));
    }

    Ok(())
}

fn verify_products(
    products: &[ResourceProduct],
    schema: &StageSchema,
    bindings: &ValidatedStageBindings,
) -> Result<(), String> {
    let mut remaining = products.to_vec();

    for (port, key) in bindings.outputs() {
        let Some(output) = schema.output(port) else {
            return Err(format!("bound output port '{port}' is missing from schema"));
        };
        let expected = ResourceProduct::create(*key, output.resource_kind());
        let Some(index) = remaining.iter().position(|product| *product == expected) else {
            return Err(format!(
                "missing create-only product for output port '{port}' bound to {key:?}"
            ));
        };

        remaining.remove(index);
    }

    if let Some(product) = remaining.first() {
        return Err(format!(
            "undeclared product for resource {:?} ({:?}, replace: {})",
            product.key, product.kind, product.replace
        ));
    }

    Ok(())
}
