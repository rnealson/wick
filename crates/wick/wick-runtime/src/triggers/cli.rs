use std::collections::HashMap;
use std::sync::Arc;
use std::{env, fmt};

use async_trait::async_trait;
use config::{AppConfiguration, TriggerDefinition};
use parking_lot::Mutex;
use serde_json::json;
use structured_output::StructuredOutput;
use tokio_stream::StreamExt;
use tracing::{Instrument, Span};
use wick_packet::{packet_stream, Entity, InherentData, Invocation};

use super::{Trigger, TriggerKind};
use crate::dev::prelude::*;
use crate::resources::Resource;
use crate::Runtime;

#[derive(Debug)]
pub(crate) struct Cli {
  done_tx: Mutex<Option<tokio::sync::oneshot::Sender<StructuredOutput>>>,
  done_rx: Mutex<Option<tokio::sync::oneshot::Receiver<StructuredOutput>>>,
}

impl Cli {
  pub(crate) fn load() -> Result<Arc<dyn Trigger + Send + Sync>, RuntimeError> {
    Ok(Arc::new(Cli::load_impl()?))
  }

  pub(crate) fn load_impl() -> Result<Cli, RuntimeError> {
    let (done_tx, done_rx) = tokio::sync::oneshot::channel();
    Ok(Self {
      done_tx: Mutex::new(Some(done_tx)),
      done_rx: Mutex::new(Some(done_rx)),
    })
  }

  async fn handle(&self, runtime: Runtime, operation: Entity, args: Vec<String>) -> Result<(), RuntimeError> {
    let is_interactive = wick_interface_cli::types::Interactive {
      stdin: atty::is(atty::Stream::Stdin),
      stdout: atty::is(atty::Stream::Stdout),
      stderr: atty::is(atty::Stream::Stderr),
    };

    let packet_stream = packet_stream!(("args", args), ("interactive", is_interactive));
    let invocation = Invocation::new(
      Entity::server("cli_channel"),
      operation,
      packet_stream,
      InherentData::unsafe_default(),
      &Span::current(),
    );

    let mut response = runtime.invoke(invocation, Default::default()).await?;
    let output = loop {
      if let Some(packet) = response.next().await {
        trace!(?packet, "trigger:cli:response");
        match packet {
          Ok(p) => {
            if p.port() == "code" && p.has_data() {
              let code: u32 = p.decode().unwrap();
              let message = if code > 0 {
                format!("Exit code: {}", code)
              } else {
                String::new()
              };
              break StructuredOutput::new(message, json!({ "code": code }));
            }
            if p.is_error() {
              let err = p.unwrap_err();
              break StructuredOutput::new(
                format!("CLI Trigger produced error, {}", err.msg()),
                json!({ "error": err.to_string() }),
              );
            }
          }
          Err(e) => {
            break StructuredOutput::new(
              format!("CLI Trigger produced error, {}", e),
              json!({ "error": e.to_string() }),
            );
          }
        }
      } else {
        break StructuredOutput::new(
          "CLI Trigger failed to return an exit code",
          json!({ "error": "CLI Trigger failed to return an exit code" }),
        );
      }
    };

    let _ = self.done_tx.lock().take().unwrap().send(output);

    Ok(())
  }
}

#[async_trait]
impl Trigger for Cli {
  async fn run(
    &self,
    name: String,
    runtime: Runtime,
    _app_config: AppConfiguration,
    config: TriggerDefinition,
    _resources: Arc<HashMap<String, Resource>>,
    span: Span,
  ) -> Result<StructuredOutput, RuntimeError> {
    let TriggerDefinition::Cli(config) = config else {
      return Err(RuntimeError::TriggerKind(Context::Trigger, TriggerKind::Cli));
    };

    let mut args: Vec<String> = env::args().collect();
    // Preserve only the arguments after `--`.
    while !args.is_empty() && &args[0] != "--" {
      args.remove(0);
    }
    if !args.is_empty() && &args[0] == "--" {
      args.remove(0);
    }

    // Insert app name as the first argument.
    args.insert(0, name);

    let target = config.operation().as_entity().unwrap();

    self.handle(runtime, target, args).instrument(span).await?;

    Ok(StructuredOutput::default())
  }

  async fn shutdown_gracefully(self) -> Result<(), RuntimeError> {
    Ok(())
  }

  async fn wait_for_done(&self) -> StructuredOutput {
    let rx = self.done_rx.lock().take().unwrap();
    rx.await.unwrap_or_default()
  }
}

impl fmt::Display for Cli {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Cli Trigger",)
  }
}
