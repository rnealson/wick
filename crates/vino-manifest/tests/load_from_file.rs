use std::env;
use std::path::PathBuf;

use tracing::debug;
use vino_manifest::error::ManifestError;
use vino_manifest::*;

#[test_env_log::test]
fn load_manifest_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/logger.yaml");
  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  assert_eq!(manifest.default_schematic, "logger");

  Ok(())
}

#[test_env_log::test]
fn load_bad_manifest_yaml() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/bad-yaml.yaml");
  let manifest = HostManifest::load_from_file(&path);
  if let Err(Error::YamlError(e)) = manifest {
    debug!("{:?}", e);
  } else {
    panic!("Should have failed")
  }

  Ok(())
}

#[test_env_log::test]
fn load_manifest_hocon() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/logger.manifest");
  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  assert_eq!(manifest.default_schematic, "logger");

  Ok(())
}

#[test_env_log::test]

fn load_manifest_env() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/logger-with-env.yaml");
  env::set_var("TEST_ENV_VAR", "load_manifest_yaml_with_env");
  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  assert_eq!(manifest.default_schematic, "logger");
  assert_eq!(
    manifest.network.schematics[0].name,
    "name_load_manifest_yaml_with_env"
  );

  let path = PathBuf::from("./tests/manifests/v0/logger-with-env.manifest");
  env::set_var("TEST_ENV_VAR", "load_manifest_hocon_env");

  let manifest = HostManifest::load_from_file(&path)?;

  let HostManifest::V0(manifest) = manifest;
  assert_eq!(manifest.default_schematic, "logger");
  assert_eq!(
    manifest.network.schematics[0].name,
    "name_load_manifest_hocon_env"
  );

  Ok(())
}

#[test_env_log::test]
fn load_bad_manifest_hocon() -> Result<(), ManifestError> {
  let path = PathBuf::from("./tests/manifests/v0/bad-hocon.manifest");
  let manifest = HostManifest::load_from_file(&path);
  if let Err(Error::HoconError(e)) = manifest {
    debug!("{:?}", e);
  } else {
    panic!("Should have failed")
  }

  Ok(())
}
