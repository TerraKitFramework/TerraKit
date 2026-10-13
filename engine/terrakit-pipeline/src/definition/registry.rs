//! Registry of discoverable stage definitions.

use std::collections::BTreeMap;

use super::{RegistryError, StageDefinition, StageSchema, StageTypeId};

/// Deterministic catalogue of available stage definitions.
#[derive(Default)]
pub struct StageRegistry {
    definitions: BTreeMap<StageTypeId, Box<dyn StageDefinition>>,
}

impl StageRegistry {
    /// Creates an empty stage registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one owned stage definition.
    pub fn register<D>(&mut self, definition: D) -> Result<(), RegistryError>
    where
        D: StageDefinition + 'static,
    {
        self.register_boxed(Box::new(definition))
    }

    /// Registers one boxed stage definition.
    pub fn register_boxed(
        &mut self,
        definition: Box<dyn StageDefinition>,
    ) -> Result<(), RegistryError> {
        let stage_type = definition.schema().type_id().clone();
        if self.definitions.contains_key(&stage_type) {
            return Err(RegistryError::DuplicateStageType { stage_type });
        }

        self.definitions.insert(stage_type, definition);
        Ok(())
    }

    /// Returns a registered stage definition by type ID.
    pub fn definition(&self, type_id: &StageTypeId) -> Option<&dyn StageDefinition> {
        self.definitions.get(type_id).map(Box::as_ref)
    }

    /// Returns registered schemas in deterministic stage type ID order.
    pub fn schemas(&self) -> impl Iterator<Item = &StageSchema> {
        self.definitions
            .values()
            .map(|definition| definition.schema())
    }

    /// Returns the number of registered definitions.
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    /// Returns true when no definitions are registered.
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }
}
