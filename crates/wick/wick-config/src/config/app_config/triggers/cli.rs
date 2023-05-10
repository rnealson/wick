use wick_asset_reference::AssetReference;

use crate::config::ComponentOperationExpression;

#[derive(Debug, Clone, PartialEq, derive_asset_container::AssetManager, property::Property)]
#[asset(asset(AssetReference))]
#[property(get(public), set(private), mut(disable))]

/// Normalized representation of a CLI trigger configuration.
pub struct CliConfig {
  pub(crate) operation: ComponentOperationExpression,
}
