use std::sync::Arc;

use structopt::StructOpt;
use tokio::sync::Mutex;
use vino_provider_wasm::provider::Provider;

use crate::Result;
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub(crate) struct ServeCommand {
  /// Path or URL to WebAssembly binary
  pub(crate) wasm: String,

  #[structopt(flatten)]
  pub(crate) logging: super::LoggingOptions,

  #[structopt(flatten)]
  pub(crate) connect: super::ConnectOptions,

  #[structopt(flatten)]
  pub(crate) pull: super::PullOptions,
}

pub(crate) async fn handle_command(opts: ServeCommand) -> Result<()> {
  logger::Logger::init(&opts.logging)?;

  debug!("Loading wasm {}", opts.wasm);
  let component =
    vino_provider_wasm::helpers::load_wasm(&opts.wasm, opts.pull.latest, &opts.pull.insecure)
      .await?;

  vino_provider_cli::init_cli(
    Arc::new(Mutex::new(Provider::new(component, 5))),
    Some(vino_provider_cli::cli::Options {
      port: opts.connect.port,
      address: opts.connect.address,
      pem: opts.connect.pem,
      ca: opts.connect.ca,
      key: opts.connect.key,
    }),
  )
  .await?;

  Ok(())
}