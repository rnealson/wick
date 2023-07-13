#![allow(missing_docs)] // delete when we move away from the `property` crate.
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use asset_container::AssetManager;

use super::LiquidJsonConfig;
use crate::config;
use crate::error::ManifestError;

#[derive(Debug, Clone, Builder, derive_asset_container::AssetManager, property::Property)]
#[property(get(public), set(public), mut(public, suffix = "_mut"))]
#[asset(asset(crate::config::AssetReference))]
#[must_use]
/// A Wick tests configuration.
///
/// A tests configuration is a collection of shareable and reusable unit tests against wick components and operations.
pub struct TestConfiguration {
  /// The name of the tests configuration.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) name: Option<String>,

  /// The source (i.e. url or file on disk) of the configuration.
  #[asset(skip)]
  #[property(skip)]
  #[builder(default)]
  pub(crate) source: Option<PathBuf>,

  /// The configuration with which to initialize the component before running tests.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) config: Option<LiquidJsonConfig>,

  /// A suite of test cases to run against component operations.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) cases: Vec<config::TestCase>,

  /// The environment this configuration has access to.
  #[asset(skip)]
  #[builder(default)]
  pub(crate) env: Option<HashMap<String, String>>,
}

impl TestConfiguration {
  /// Set the source location of the configuration.
  pub fn set_source(&mut self, source: &Path) {
    let mut source = source.to_path_buf();
    self.source = Some(source.clone());
    // Source is (should be) a file, so pop the filename before setting the baseurl.
    if !source.is_dir() {
      source.pop();
    }
    self.set_baseurl(&source);
  }

  /// Validate this configuration is good.
  pub fn validate(&self) -> Result<(), ManifestError> {
    /* placeholder */
    Ok(())
  }
}
