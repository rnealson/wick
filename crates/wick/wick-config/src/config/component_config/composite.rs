use std::collections::HashMap;

use wick_interface_types::TypeDefinition;

use crate::common::BoundInterface;
use crate::config::common::bindings::BoundComponent;
use crate::config::common::flow_definition::FlowOperation;

#[derive(Debug, Default, Clone, derive_asset_container::AssetManager)]
#[asset(crate::config::AssetReference)]
#[must_use]
/// The internal representation of a Wick manifest.
pub struct CompositeComponentImplementation {
  #[asset(skip)]
  pub(crate) types: Vec<TypeDefinition>,
  pub(crate) import: HashMap<String, BoundComponent>,
  #[asset(skip)]
  pub(crate) operations: HashMap<String, FlowOperation>,
  #[asset(skip)]
  pub(crate) requires: HashMap<String, BoundInterface>,
}

impl CompositeComponentImplementation {
  /// Add a [BoundComponent] to the configuration.
  pub fn add_import(mut self, component: BoundComponent) -> Self {
    self.import.insert(component.id.clone(), component);
    self
  }

  /// Get the types used by this component.
  pub fn types(&self) -> &[TypeDefinition] {
    &self.types
  }

  #[must_use]
  /// Get the components imported by this [CompositeComponentConfiguration].
  pub fn components(&self) -> &HashMap<String, BoundComponent> {
    &self.import
  }

  #[must_use]
  /// Get an imported component by ID.
  pub fn component(&self, id: &str) -> Option<&BoundComponent> {
    self.import.iter().find(|(k, _)| *k == id).map(|(_, v)| v)
  }

  #[must_use]
  /// Get a map of [FlowOperation]s from the [CompositeComponentConfiguration]
  pub fn operations(&self) -> &HashMap<String, FlowOperation> {
    &self.operations
  }

  /// Get a [FlowOperation] by name.
  #[must_use]
  pub fn flow(&self, name: &str) -> Option<&FlowOperation> {
    self.operations.iter().find(|(n, _)| name == *n).map(|(_, v)| v)
  }

  /// Get the required interfaces for this component.
  #[must_use]
  pub fn requires(&self) -> &HashMap<String, BoundInterface> {
    &self.requires
  }

  /// Get the signature of the component as defined by the manifest.
  #[must_use]
  pub fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    self.operations.values().cloned().map(Into::into).collect()
  }
}

impl From<FlowOperation> for wick_interface_types::OperationSignature {
  fn from(operation: FlowOperation) -> Self {
    Self {
      name: operation.name,
      inputs: operation.inputs,
      outputs: operation.outputs,
    }
  }
}