use flow_component::{Component, ComponentError, Operation, RuntimeCallback};
use serde_json::Value;
use wick_interface_types::{
  ComponentSignature,
  Field,
  OperationSignature,
  StructSignature,
  TypeDefinition,
  TypeSignature,
};
use wick_packet::{Invocation, PacketStream, StreamMap};

use crate::constants::*;
use crate::graph::types::Network;
use crate::interpreter::components::dyn_component_id;
use crate::BoxFuture;

// mod merge;
mod sender;

#[derive(Debug)]
pub(crate) struct CoreCollection {
  signature: ComponentSignature,
}

impl CoreCollection {
  pub(crate) fn new(graph: &Network) -> Self {
    let mut signature = ComponentSignature::new(NS_CORE)
      .version("0.0.0")
      .add_operation(OperationSignature::new(CORE_ID_SENDER).add_output("output", TypeSignature::Object));

    for schematic in graph.schematics() {
      for component in schematic.nodes() {
        // only handle core:: components
        if component.cref().component_id() != NS_CORE {
          continue;
        }
        // set up dynamic merge components
        if component.cref().name() == CORE_ID_MERGE {
          assert!(
            component.has_data(),
            "Dynamic merge component ({}, instance {}) must be configured with its expected inputs.",
            CORE_ID_MERGE,
            component.id()
          );

          let result = serde_json::from_value::<OperationSignature>(component.data().clone().unwrap());
          if let Err(e) = result {
            panic!("Configuration for dynamic merge component invalid: {}", e);
          }
          let id = dyn_component_id(CORE_ID_MERGE, schematic.name(), component.id());
          let mut component_signature = result.unwrap();
          let output_type = Vec::new();
          let mut output_signature = StructSignature::new(&id, output_type);
          for field in &component_signature.inputs {
            output_signature.fields.push(field.clone());
          }
          signature.types.push(TypeDefinition::Struct(output_signature));

          component_signature
            .outputs
            .push(Field::new("output", TypeSignature::Ref { reference: id.clone() }));
          debug!(%id,"adding dynamic component");
          signature.operations.push(component_signature);
        }
      }
    }

    trace!(?signature, "core signature");

    Self { signature }
  }
}

impl Component for CoreCollection {
  fn handle(
    &self,
    invocation: Invocation,
    _stream: PacketStream,
    data: Option<Value>,
    _callback: std::sync::Arc<RuntimeCallback>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    trace!(target = %invocation.target, namespace = NS_CORE);

    let task = async move {
      match invocation.target.operation_id() {
        CORE_ID_SENDER => {
          let map = StreamMap::default();
          sender::SenderOperation::default().handle(map, data).await
        }
        // TODO re-evaluate merge component
        // CORE_ID_MERGE => merge::MergeComponent::default().handle(invocation.payload, data).await,
        _ => {
          panic!("Core operation {} not handled.", invocation.target.operation_id());
        }
      }
    };
    Box::pin(task)
  }

  fn list(&self) -> &ComponentSignature {
    &self.signature
  }
}