use serde::de::{IgnoredAny, SeqAccess, Visitor};
use serde::Deserializer;
use serde_json::Value;

use crate::config;

/// A reference to an operation.
#[derive(Debug, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(config::AssetReference)]

pub struct ComponentOperationExpression {
  /// The operation ID.
  #[asset(skip)]
  pub(crate) operation: String,
  /// The component referenced by identifier or anonymously.
  pub(crate) component: ComponentDefinition,
}

impl ComponentOperationExpression {
  /// Create a new [ComponentOperationExpression] with specified operation and component.
  pub fn new(operation: impl AsRef<str>, component: ComponentDefinition) -> Self {
    Self {
      operation: operation.as_ref().to_owned(),
      component,
    }
  }

  /// Returns the operation ID.
  #[must_use]
  pub fn operation(&self) -> &str {
    &self.operation
  }

  /// Returns the component definition.
  pub fn component(&self) -> &ComponentDefinition {
    &self.component
  }
}

impl std::str::FromStr for ComponentOperationExpression {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split("::");

    let operation = parts
      .next()
      .ok_or_else(|| crate::Error::InvalidOperationExpression(s.to_owned()))?
      .to_owned();
    let component = parts
      .next()
      .ok_or_else(|| crate::Error::InvalidOperationExpression(s.to_owned()))?
      .to_owned();

    Ok(Self {
      operation,
      component: ComponentDefinition::Reference(ComponentReference { id: component }),
    })
  }
}

#[derive(Debug, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(config::AssetReference)]
/// A definition of a Wick Collection with its namespace, how to retrieve or access it and its configuration.
#[must_use]
pub struct BoundComponent {
  /// The namespace to reference the collection's components on.
  #[asset(skip)]
  pub id: String,
  /// The kind/type of the collection.
  pub kind: ComponentDefinition,
}

impl BoundComponent {
  /// Create a new [CollectionDefinition] with specified name and type.
  pub fn new(name: impl AsRef<str>, kind: ComponentDefinition) -> Self {
    Self {
      id: name.as_ref().to_owned(),
      kind,
    }
  }

  /// Get the configuration object for the collection.
  #[must_use]
  pub fn config(&self) -> Option<&Value> {
    match &self.kind {
      ComponentDefinition::Native(_) => None,
      #[allow(deprecated)]
      ComponentDefinition::Wasm(v) => Some(&v.config),
      ComponentDefinition::GrpcUrl(v) => Some(&v.config),
      ComponentDefinition::Manifest(v) => Some(&v.config),
      ComponentDefinition::Reference(_) => panic!("Cannot get config for a reference"),
    }
  }
}

#[derive(Debug, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(config::AssetReference)]
/// The kinds of collections that can operate in a flow.
#[must_use]
pub enum ComponentDefinition {
  #[doc(hidden)]
  #[asset(skip)]
  Native(NativeComponent),
  /// WebAssembly Collections.
  #[deprecated(note = "Use ManifestComponent instead")]
  Wasm(WasmComponent),
  /// A component reference.
  #[asset(skip)]
  Reference(ComponentReference),
  /// Separate microservices that Wick can connect to.
  #[asset(skip)]
  GrpcUrl(GrpcUrlComponent),
  /// External manifests.
  Manifest(ManifestComponent),
}

#[derive(Debug, Clone, PartialEq)]
/// A reference to a component by id.
pub struct ComponentReference {
  pub(crate) id: String,
}

impl ComponentReference {
  /// Get the id of the referenced component.
  #[must_use]
  pub fn id(&self) -> &str {
    &self.id
  }
}

impl ComponentDefinition {
  /// Returns true if the definition is a reference to another component.
  #[must_use]
  pub fn is_reference(&self) -> bool {
    matches!(self, ComponentDefinition::Reference(_))
  }
}

/// A native collection compiled and built in to the runtime.
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_copy_implementations)]
pub struct NativeComponent {}

/// A WebAssembly collection.
#[derive(Debug, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(config::AssetReference)]
pub struct WasmComponent {
  /// The OCI reference/local path of the collection.
  pub reference: config::AssetReference,
  /// The configuration for the collection
  #[asset(skip)]
  pub config: Value,
  /// Permissions for this collection
  #[asset(skip)]
  pub permissions: Permissions,
}

/// The permissions object for a collection
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Permissions {
  /// A map of directories (Note: TO -> FROM) to expose to the collection.
  #[serde(default)]
  pub dirs: std::collections::HashMap<String, String>,
}

/// A collection exposed as an external microservice.
#[derive(Debug, Clone, PartialEq)]
pub struct GrpcUrlComponent {
  /// The URL to connect to .
  pub url: String,
  /// The configuration for the collection
  pub config: Value,
}

/// A separate Wick manifest to use as a collection.
#[derive(Debug, Clone, PartialEq, derive_assets::AssetManager)]
#[asset(config::AssetReference)]
pub struct ManifestComponent {
  /// The OCI reference/local path of the manifest to use as a collection.
  pub reference: config::AssetReference,
  /// The configuration for the collection
  #[asset(skip)]
  pub config: Value,
}

#[derive(Default, Debug)]
struct StringPair(String, String);

impl<'de> serde::Deserialize<'de> for StringPair {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct StringPairVisitor;

    impl<'de> Visitor<'de> for StringPairVisitor {
      type Value = StringPair;

      fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a String pair")
      }

      fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
      where
        V: SeqAccess<'de>,
      {
        let s = seq
          .next_element()?
          .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let n = seq
          .next_element()?
          .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

        // This is very important!
        while matches!(seq.next_element()?, Some(IgnoredAny)) {
          // Ignore rest
        }

        Ok(StringPair(s, n))
      }
    }

    deserializer.deserialize_seq(StringPairVisitor)
  }
}
