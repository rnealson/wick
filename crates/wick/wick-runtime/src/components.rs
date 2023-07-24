pub(crate) mod component_service;
pub(crate) mod engine_component;
pub(crate) mod error;
pub(crate) mod validation;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use flow_component::{Component, RuntimeCallback};
use flow_graph_interpreter::NamespaceHandler;
use seeded_random::{Random, Seed};
use tracing::Instrument;
use uuid::Uuid;
use wick_component_wasm::component::{ComponentSetupBuilder, WasmComponent};
use wick_component_wasm::error::LinkError;
use wick_config::config::components::ManifestComponent;
use wick_config::config::{
  BoundInterface,
  Metadata,
  Permissions,
  PermissionsBuilder,
  ResourceDefinition,
  WasmComponentImplementation,
};
use wick_config::{AssetReference, FetchOptions, Resolver, WickConfiguration};
use wick_packet::validation::expect_configuration_matches;
use wick_packet::{Entity, Invocation, RuntimeConfig};

use self::component_service::NativeComponentService;
use self::validation::expect_signature_match;
use crate::dev::prelude::*;
use crate::dispatch::engine_invoke_async;
use crate::runtime_service::{init_child, ChildInit};
use crate::wasmtime::WASMTIME_ENGINE;
use crate::BoxFuture;

pub(crate) trait InvocationHandler {
  fn get_signature(&self) -> Result<ComponentSignature>;
  fn invoke(&self, msg: Invocation, config: Option<RuntimeConfig>) -> Result<BoxFuture<Result<InvocationResponse>>>;
}

type Result<T> = std::result::Result<T, ComponentError>;

type ComponentInitResult = std::result::Result<NamespaceHandler, EngineError>;

pub(crate) async fn init_wasm_component(
  reference: &AssetReference,
  namespace: String,
  opts: ChildInit,
  permissions: Option<Permissions>,
  provided: HashMap<String, String>,
) -> ComponentInitResult {
  opts
    .span
    .in_scope(|| trace!(namespace = %namespace, ?opts, ?permissions, "registering wasm component"));

  let mut options = FetchOptions::default();
  options
    .set_allow_latest(opts.allow_latest)
    .set_allow_insecure(opts.allowed_insecure.clone());
  let asset = reference.with_options(options);

  let setup = ComponentSetupBuilder::default()
    .engine(WASMTIME_ENGINE.clone())
    .permissions(permissions)
    .config(opts.root_config)
    .callback(Some(make_link_callback(opts.runtime_id)))
    .provided(provided)
    .build()
    .unwrap();

  let component = WasmComponent::try_load(&namespace, asset, setup, opts.span).await?;

  let component = Arc::new(component);

  let service = NativeComponentService::new(component);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

pub(crate) async fn init_wasm_impl_component(
  kind: &WasmComponentImplementation,
  namespace: String,
  opts: ChildInit,
  permissions: Option<Permissions>,
  provided: HashMap<String, String>,
) -> ComponentInitResult {
  init_wasm_component(kind.reference(), namespace, opts, permissions, provided).await
}

pub(crate) fn make_link_callback(engine_id: Uuid) -> Arc<RuntimeCallback> {
  Arc::new(move |compref, op, stream, inherent, config, span| {
    let origin_url = compref.get_origin_url();
    let target_id = compref.get_target_id().to_owned();
    let invocation = compref.to_invocation(&op, stream, inherent, span);
    invocation.trace(|| {
      debug!(
        origin = %origin_url,
        target = %target_id,
        engine_id = %engine_id,
        config = ?config,
        "link_call"
      );
    });
    Box::pin(async move {
      {
        let result = engine_invoke_async(engine_id, invocation, config)
          .await
          .map_err(|e| flow_component::ComponentError::new(LinkError::CallFailure(e.to_string())))?;
        Ok(result)
      }
    })
  })
}

pub(crate) async fn init_manifest_component(
  kind: &ManifestComponent,
  id: String,
  opts: ChildInit,
) -> ComponentInitResult {
  let span = opts.span.clone();
  span.in_scope(|| trace!(namespace = %id, ?opts, "registering wick component"));

  let mut options = FetchOptions::default();

  options
    .set_allow_latest(opts.allow_latest)
    .set_allow_insecure(opts.allowed_insecure.clone());

  let mut builder = WickConfiguration::fetch(kind.reference().path()?.to_string_lossy(), options)
    .instrument(span.clone())
    .await?;
  builder.set_root_config(opts.root_config.clone());
  let manifest = builder.finish()?.try_component_config()?;

  let requires = manifest.requires();
  let provided =
    generate_provides(requires, kind.provide()).map_err(|e| EngineError::ComponentInit(id.clone(), e.to_string()))?;
  init_component_implementation(&manifest, id, opts, provided).await
}

pub(crate) async fn init_component_implementation(
  manifest: &ComponentConfiguration,
  id: String,
  mut opts: ChildInit,
  provided: HashMap<String, String>,
) -> ComponentInitResult {
  let span = opts.span.clone();
  span.in_scope(|| {
    debug!(%id,"validating configuration for wick component");
    expect_configuration_matches(&id, opts.root_config.as_ref(), manifest.config()).map_err(EngineError::Setup)
  })?;

  let rng = Random::from_seed(opts.rng_seed);
  opts.rng_seed = rng.seed();
  let uuid = rng.uuid();
  let metadata = manifest.metadata();
  match manifest.component() {
    config::ComponentImplementation::Wasm(wasmimpl) => {
      let mut dirs = HashMap::new();
      for volume in wasmimpl.volumes() {
        let resource = (opts.resolver)(volume.resource())?.try_resource()?.try_volume()?;
        dirs.insert(volume.path().to_owned(), resource.path()?.to_string_lossy().to_string());
      }
      let perms = if !dirs.is_empty() {
        let mut perms = PermissionsBuilder::default();
        perms.dirs(dirs);
        Some(perms.build().unwrap())
      } else {
        None
      };
      let comp = init_wasm_impl_component(wasmimpl, id.clone(), opts, perms, provided).await?;
      let signed_sig = comp.component().signature();
      let manifest_sig = manifest.signature()?;
      span.in_scope(|| {
        expect_signature_match(
          Some(&PathBuf::from(&id)),
          signed_sig,
          Some(&PathBuf::from(wasmimpl.reference().location())),
          &manifest_sig,
        )
      })?;
      Ok(comp)
    }
    config::ComponentImplementation::Composite(_) => {
      let _engine = init_child(uuid, manifest.clone(), Some(id.clone()), opts).await?;

      let component = Arc::new(engine_component::EngineComponent::new(uuid));
      let service = NativeComponentService::new(component);
      Ok(NamespaceHandler::new(id, Box::new(service)))
    }
    config::ComponentImplementation::Sql(c) => {
      init_hlc_component(
        id,
        opts.root_config.clone(),
        metadata.cloned(),
        wick_config::config::HighLevelComponent::Sql(c.clone()),
        manifest.resolver(),
      )
      .await
    }
    config::ComponentImplementation::HttpClient(c) => {
      init_hlc_component(
        id,
        opts.root_config.clone(),
        metadata.cloned(),
        wick_config::config::HighLevelComponent::HttpClient(c.clone()),
        manifest.resolver(),
      )
      .await
    }
  }
}

fn generate_provides(
  requires: &HashMap<String, BoundInterface>,
  provides: &HashMap<String, String>,
) -> Result<HashMap<String, String>> {
  let mut provide = HashMap::new();
  #[allow(clippy::for_kv_map)] // silencing clippy to keep context for the TODO below.
  for (id, _interface) in requires {
    if let Some(provided) = provides.get(id) {
      provide.insert(id.clone(), Entity::component(provided).url());
      // TODO: validate interfaces against what was provided.
    } else {
      return Err(ComponentError::UnsatisfiedRequirement(id.clone()));
    }
  }
  Ok(provide)
}

pub(crate) fn initialize_native_component(namespace: String, seed: Seed) -> ComponentInitResult {
  let collection = Arc::new(wick_stdlib::Collection::new(seed));
  let service = NativeComponentService::new(collection);

  Ok(NamespaceHandler::new(namespace, Box::new(service)))
}

pub(crate) async fn init_hlc_component(
  id: String,
  root_config: Option<RuntimeConfig>,
  metadata: Option<Metadata>,
  component: wick_config::config::HighLevelComponent,
  resolver: Box<Resolver>,
) -> ComponentInitResult {
  let mut comp: Box<dyn Component + Send + Sync> = match component {
    config::HighLevelComponent::Sql(comp) => {
      Box::new(wick_sql::SqlComponent::new(comp, root_config, metadata, &resolver)?)
    }
    config::HighLevelComponent::HttpClient(comp) => Box::new(wick_http_client::HttpClientComponent::new(
      comp,
      root_config,
      metadata,
      &resolver,
    )?),
  };
  comp.init().await.map_err(EngineError::NativeComponent)?;
  Ok(NamespaceHandler::new(id, comp))
}
