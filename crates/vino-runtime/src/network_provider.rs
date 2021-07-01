use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_rpc::port::PortPacket;
use vino_rpc::{
  BoxedPacketStream,
  ExecutionStatistics,
  HostedType,
  RpcHandler,
  RpcResult,
  Statistics,
};

use crate::dev::prelude::*;
use crate::network_service::ListSchematics;

#[derive(Debug, Default)]
struct State {
  documents: HashMap<String, String>,
  collections: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct Provider {
  network_id: String,
  context: Arc<Mutex<State>>,
}

impl Provider {
  #[must_use]
  pub fn new(network_id: String) -> Self {
    Self {
      network_id,
      context: Arc::new(Mutex::new(State::default())),
    }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn request(
    &self,
    _inv_id: String,
    component: String,
    payload: HashMap<String, Vec<u8>>,
  ) -> RpcResult<BoxedPacketStream> {
    let addr = NetworkService::for_id(&self.network_id);
    let result: InvocationResponse = addr
      .send(Invocation {
        origin: VinoEntity::Schematic("<system>".to_owned()),
        target: VinoEntity::Schematic(component),
        msg: MessageTransport::MultiBytes(payload),
        id: get_uuid(),
        tx_id: get_uuid(),
        encoded_claims: "".to_owned(),
        network_id: get_uuid(),
      })
      .await?;
    match result {
      InvocationResponse::Success { .. } => unreachable!(),
      InvocationResponse::Stream { rx, .. } => Ok(Box::pin(rx.map(|output| PortPacket {
        port: output.port,
        packet: output.payload,
      }))),
      InvocationResponse::Error { msg, .. } => Err(Box::new(Error::SchematicError(format!(
        "Invocation failed: {}",
        msg
      )))),
    }
  }

  async fn list_registered(&self) -> RpcResult<Vec<HostedType>> {
    let addr = NetworkService::for_id(&self.network_id);
    let schematics = addr.send(ListSchematics {}).await??;
    let hosted_types = schematics.into_iter().map(HostedType::Schematic).collect();

    Ok(hosted_types)
  }

  async fn report_statistics(&self, id: Option<String>) -> RpcResult<Vec<Statistics>> {
    // TODO Dummy implementation
    if id.is_some() {
      Ok(vec![Statistics {
        num_calls: 1,
        execution_duration: ExecutionStatistics {
          max_time: 0,
          min_time: 0,
          average: 0,
        },
      }])
    } else {
      Ok(vec![Statistics {
        num_calls: 0,
        execution_duration: ExecutionStatistics {
          max_time: 0,
          min_time: 0,
          average: 0,
        },
      }])
    }
  }
}

#[cfg(test)]
mod tests {
  use maplit::hashmap;

  use super::*;
  use crate::test::prelude::*;

  async fn request_log(provider: &Provider, data: &str) -> Result<String> {
    let job_payload = hashmap! {
      "input".to_owned() => mp_serialize(data)?,
    };
    let invocation_id = "INVOCATION_ID";

    let mut outputs = provider
      .request(invocation_id.to_owned(), "simple".to_owned(), job_payload)
      .await?;
    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.packet);
    let output_data: String = output.packet.try_into()?;

    println!("doc_id: {:?}", output_data);
    assert_eq!(output_data, data);
    Ok(output_data)
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_request_log() -> Result<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/simple.yaml").await?;

    let provider = Provider::new(network_id);
    let user_data = "string to log";
    let result = request_log(&provider, user_data).await?;
    print!("Result: {}", result);

    Ok(())
  }

  #[test_env_log::test(actix_rt::test)]
  async fn test_list() -> Result<()> {
    let (_, network_id) = init_network_from_yaml("./manifests/simple.yaml").await?;
    let provider = Provider::new(network_id);
    let list = provider.list_registered().await?;
    println!("components on network : {:?}", list);
    assert_eq!(list.len(), 1);

    Ok(())
  }
}