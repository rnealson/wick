use flow_expression_parser::ast::{
  ConnectionExpression,
  ConnectionTargetExpression,
  FlowExpression,
  InstancePort,
  InstanceTarget,
};
use wick_config::config::components::ComponentConfig;
use wick_config::config::{ComponentImplementation, FlowOperationBuilder};
use wick_config::error::ManifestError;
use wick_config::*;

use crate::utils::{load, load_app, load_composite};
mod utils;

#[test_logger::test(tokio::test)]
async fn test_basics() -> Result<(), ManifestError> {
  let component = load_composite("./tests/manifests/v1/logger.yaml").await?;

  assert_eq!(component.flow("logger").map(|s| s.instances().len()), Some(2));

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_types() -> Result<(), ManifestError> {
  let types = load("./tests/manifests/v1/http-types.yaml").await?.try_types_config()?;
  assert_eq!(types.types().len(), 6);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_tests() -> Result<(), ManifestError> {
  let tests = load("./tests/manifests/v1/tests.yaml").await?.try_test_config()?;

  assert_eq!(tests.cases().len(), 1);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_operations() -> Result<(), ManifestError> {
  let component = load_composite("./tests/manifests/v1/operations.yaml").await?;
  assert_eq!(component.operations().len(), 1);

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_component() -> Result<(), ManifestError> {
  let component = load("./tests/manifests/v1/component.yaml")
    .await?
    .try_component_config()?;

  assert!(matches!(component.component().kind(), config::ComponentKind::Wasm));

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn test_component_singular_input_field() -> Result<(), ManifestError> {
  let component = load("./tests/manifests/v1/component-old.yaml")
    .await?
    .try_component_config()?;

  assert!(matches!(component.component().kind(), config::ComponentKind::Wasm));

  Ok(())
}
#[test_logger::test(tokio::test)]
async fn test_component_extended() -> Result<(), ManifestError> {
  let component = load("./tests/manifests/v1/component-extended.yaml")
    .await?
    .try_component_config()?;

  if let ComponentImplementation::Composite(config) = component.component() {
    let op = FlowOperationBuilder::default()
      .name("test")
      .expressions(vec![FlowExpression::connection(ConnectionExpression::new(
        ConnectionTargetExpression::new(InstanceTarget::Input, InstancePort::named("input")),
        ConnectionTargetExpression::new(InstanceTarget::Output, InstancePort::named("output")),
      ))])
      .build()
      .unwrap();
    assert_eq!(config.operations(), &[op])
  } else {
    panic!("Wrong component type");
  }

  Ok(())
}

#[test_logger::test(tokio::test)]
async fn regression_issue_42() -> Result<(), ManifestError> {
  let component = load_app("./tests/manifests/v1/template-expansion.yaml").await?;
  println!("{:?}", component);
  let coll = component.import().get(0).unwrap();
  let value: String = coll.kind().config().unwrap().coerce_key("pwd").unwrap();
  let expected = std::env::var("CARGO_MANIFEST_DIR").unwrap();

  assert_eq!(value, expected);
  Ok(())
}
