#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::borrow::Cow;
use std::collections::HashMap;

use wick_interface_types::OperationSignatures;

use super::{ComponentConfig, OperationConfig};
use crate::config::{self, Codec, HttpMethod};

#[derive(
  Debug,
  Clone,
  derive_builder::Builder,
  PartialEq,
  derive_asset_container::AssetManager,
  property::Property,
  serde::Serialize,
)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(config::AssetReference))]
#[builder(setter(into))]
#[must_use]
/// A component made out of other components
pub struct HttpClientComponentConfig {
  /// The URL base to use.
  #[asset(skip)]
  pub(crate) resource: String,

  /// The configuration for the component.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) config: Vec<wick_interface_types::Field>,

  /// The codec to use when encoding/decoding data.
  #[asset(skip)]
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) codec: Option<Codec>,

  /// A list of operations to expose on this component.
  #[asset(skip)]
  #[builder(default)]
  #[property(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) operations: Vec<HttpClientOperationDefinition>,
}

impl HttpClientComponentConfig {}

impl OperationSignatures for HttpClientComponentConfig {
  fn operation_signatures(&self) -> Vec<wick_interface_types::OperationSignature> {
    let codec = self.codec;
    self
      .operations
      .clone()
      .into_iter()
      .map(|mut op| {
        op.codec = op.codec.or(codec);
        op
      })
      .map(Into::into)
      .collect()
  }
}

impl ComponentConfig for HttpClientComponentConfig {
  type Operation = HttpClientOperationDefinition;

  fn operations(&self) -> &[Self::Operation] {
    &self.operations
  }

  fn operations_mut(&mut self) -> &mut Vec<Self::Operation> {
    &mut self.operations
  }
}

impl OperationConfig for HttpClientOperationDefinition {
  fn name(&self) -> &str {
    &self.name
  }

  fn inputs(&self) -> Cow<Vec<wick_interface_types::Field>> {
    Cow::Borrowed(&self.inputs)
  }

  fn outputs(&self) -> Cow<Vec<wick_interface_types::Field>> {
    Cow::Owned(vec![
      // TODO: support actual HTTP Response type.
      wick_interface_types::Field::new("response", wick_interface_types::Type::Object),
      wick_interface_types::Field::new(
        "body",
        match self.codec {
          Some(Codec::Json) => wick_interface_types::Type::Object,
          Some(Codec::Raw) => wick_interface_types::Type::Bytes,
          Some(Codec::FormData) => wick_interface_types::Type::Object,
          Some(Codec::Text) => wick_interface_types::Type::Object,
          None => wick_interface_types::Type::Object,
        },
      ),
    ])
  }
}

impl From<HttpClientOperationDefinition> for wick_interface_types::OperationSignature {
  fn from(operation: HttpClientOperationDefinition) -> Self {
    Self::new(
      operation.name,
      operation.inputs,
      vec![
        // TODO: support actual HTTP Response type.
        wick_interface_types::Field::new("response", wick_interface_types::Type::Object),
        wick_interface_types::Field::new(
          "body",
          match operation.codec {
            Some(Codec::Json) => wick_interface_types::Type::Object,
            Some(Codec::Raw) => wick_interface_types::Type::Bytes,
            Some(Codec::FormData) => wick_interface_types::Type::Object,
            Some(Codec::Text) => wick_interface_types::Type::Object,
            None => wick_interface_types::Type::Object,
          },
        ),
      ],
      operation.config,
    )
  }
}

#[derive(Debug, Clone, derive_builder::Builder, PartialEq, property::Property, serde::Serialize)]
#[property(get(public), set(private), mut(disable))]
#[builder(setter(into))]
#[must_use]
/// An operation whose implementation is an HTTP request.
pub struct HttpClientOperationDefinition {
  /// The name of the operation.
  #[property(skip)]
  pub(crate) name: String,

  /// The configuration the operation needs.
  #[builder(default)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) config: Vec<wick_interface_types::Field>,

  /// Types of the inputs to the operation.
  #[property(skip)]
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub(crate) inputs: Vec<wick_interface_types::Field>,

  /// The path to append to our base URL, processed as a liquid template with each input as part of the template data.
  pub(crate) path: String,

  /// The codec to use when encoding/decoding data.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) codec: Option<Codec>,

  /// The body to send with the request.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) body: Option<liquid_json::LiquidJsonValue>,

  /// The headers to send with the request.
  #[builder(default)]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub(crate) headers: Option<HashMap<String, Vec<String>>>,

  /// The HTTP method to use.
  pub(crate) method: HttpMethod,
}

impl HttpClientOperationDefinition {
  /// Create a new GET operation.
  #[must_use]
  pub fn new_get(
    name: &str,
    path: &str,
    inputs: Vec<wick_interface_types::Field>,
    headers: Option<HashMap<String, Vec<String>>>,
  ) -> HttpClientOperationDefinitionBuilder {
    let mut builder = HttpClientOperationDefinitionBuilder::default();
    builder
      .name(name)
      .path(path)
      .inputs(inputs)
      .headers(headers)
      .method(HttpMethod::Get);
    builder
  }

  /// Create a new POST operation.
  #[must_use]
  pub fn new_post(
    name: &str,
    path: &str,
    inputs: Vec<wick_interface_types::Field>,
    body: Option<liquid_json::LiquidJsonValue>,
    headers: Option<HashMap<String, Vec<String>>>,
  ) -> HttpClientOperationDefinitionBuilder {
    let mut builder = HttpClientOperationDefinitionBuilder::default();
    builder
      .name(name)
      .path(path)
      .inputs(inputs)
      .body(body)
      .headers(headers)
      .method(HttpMethod::Post);
    builder
  }

  /// Create a new PUT operation.
  #[must_use]
  pub fn new_put(
    name: &str,
    path: &str,
    inputs: Vec<wick_interface_types::Field>,
    body: Option<liquid_json::LiquidJsonValue>,
    headers: Option<HashMap<String, Vec<String>>>,
  ) -> HttpClientOperationDefinitionBuilder {
    let mut builder = HttpClientOperationDefinitionBuilder::default();
    builder
      .name(name)
      .path(path)
      .inputs(inputs)
      .body(body)
      .headers(headers)
      .method(HttpMethod::Put);
    builder
  }

  /// Create a new DELETE operation.
  #[must_use]
  pub fn new_delete(
    name: &str,
    path: &str,
    inputs: Vec<wick_interface_types::Field>,
    body: Option<liquid_json::LiquidJsonValue>,
    headers: Option<HashMap<String, Vec<String>>>,
  ) -> HttpClientOperationDefinitionBuilder {
    let mut builder = HttpClientOperationDefinitionBuilder::default();
    builder
      .name(name)
      .path(path)
      .inputs(inputs)
      .body(body)
      .headers(headers)
      .method(HttpMethod::Delete);
    builder
  }
}
