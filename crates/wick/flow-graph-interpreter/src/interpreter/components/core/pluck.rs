use anyhow::anyhow;
use flow_component::{ComponentError, Context, Operation, RenderConfiguration};
use futures::{FutureExt, StreamExt};
use serde_json::Value;
use wick_interface_types::{operation, OperationSignature};
use wick_packet::{Invocation, Packet, PacketStream, RuntimeConfig};

use crate::BoxFuture;
pub(crate) struct Op {
  signature: OperationSignature,
}

impl std::fmt::Debug for Op {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct(Op::ID).field("signature", &self.signature).finish()
  }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub(crate) struct Config {
  #[serde(alias = "field")]
  field: Vec<String>,
}

impl crate::graph::NodeDecorator for Op {
  fn decorate(node: &mut crate::graph::types::Node) -> Result<(), String> {
    node.add_input("input");
    node.add_output("output");
    Ok(())
  }
}

impl Op {
  pub(crate) fn new() -> Self {
    Self {
      signature: operation!(Op::ID=>{
        config: {
          "path" => "string[]"
        },
        inputs: {
          "input" => "object"
        },
        outputs: {
          "output" => "object"
        },
      }),
    }
  }
}

fn _pluck<'a>(val: &'a Value, path: &[String], idx: usize) -> Option<&'a Value> {
  if idx == path.len() {
    Some(val)
  } else {
    let part = &path[idx];
    match val {
      Value::Object(map) => map.get(part).and_then(|next| _pluck(next, path, idx + 1)),
      Value::Array(list) => {
        let i: Result<usize, _> = part.parse();
        i.map_or(None, |i| list.get(i).and_then(|next| _pluck(next, path, idx + 1)))
      }
      _ => None,
    }
  }
}

fn pluck<'a>(val: &'a Value, path: &[String]) -> Option<&'a Value> {
  _pluck(val, path, 0)
}

impl Operation for Op {
  const ID: &'static str = "pluck";
  type Config = Config;

  fn handle(
    &self,
    invocation: Invocation,
    context: Context<Self::Config>,
  ) -> BoxFuture<Result<PacketStream, ComponentError>> {
    // let mut map = StreamMap::from_stream(invocation.packets, self.input_names(&context.config));
    let field = context.config.field.clone();
    let stream = invocation.into_stream();
    let mapped = stream.filter_map(move |next| {
      let a = next.map_or_else(
        |e| Some(Err(e)),
        |packet| {
          if packet.port() != "input" {
            return None;
          }
          if packet.has_data() {
            Some(packet.decode_value().map(|obj| {
              pluck(&obj, &field).map_or_else(
                || {
                  Packet::err(
                    "output",
                    format!("could not retrieve data from object path [{}]", field.join(",")),
                  )
                },
                |value| Packet::encode("output", value),
              )
            }))
          } else {
            Some(Ok(packet.set_port("output")))
          }
        },
      );
      futures::future::ready(a)
    });
    async move { Ok(PacketStream::new(mapped)) }.boxed()
  }

  fn get_signature(&self, _config: Option<&Self::Config>) -> &OperationSignature {
    &self.signature
  }

  fn input_names(&self, _config: &Self::Config) -> Vec<String> {
    self.signature.inputs.iter().map(|n| n.name.clone()).collect()
  }
}

impl RenderConfiguration for Op {
  type Config = Config;
  type ConfigSource = RuntimeConfig;

  fn decode_config(data: Option<Self::ConfigSource>) -> Result<Self::Config, ComponentError> {
    let config =
      data.ok_or_else(|| anyhow!("Pluck component requires configuration, please specify configuration."))?;

    for (k, v) in config {
      if k == "field" {
        let field: String = serde_json::from_value(v)?;
        warn!("pluck should be configured with 'path' as an array of strings, 'field' is deprecated and will be removed in a future release.");
        return Ok(Self::Config {
          field: field.split('.').map(|s| s.to_owned()).collect(),
        });
      }
      if k == "path" {
        let field: Vec<String> = serde_json::from_value(v)?;
        return Ok(Self::Config { field });
      }
    }
    Err(anyhow!("invalid configuration for pluck, 'path' field is required",))
  }
}

#[cfg(test)]
mod test {
  use std::collections::HashMap;

  use anyhow::Result;
  use flow_component::panic_callback;
  use serde_json::json;
  use wick_packet::{packet_stream, Entity, InherentData};

  use super::*;

  #[tokio::test]
  async fn test_deprecated() -> Result<()> {
    let op = Op::new();
    let config = HashMap::from([("field".to_owned(), json!("pluck_this"))]);
    let config = Op::decode_config(Some(config.into()))?;

    let stream = packet_stream!((
      "input",
      serde_json::json!({
        "pluck_this": "hello",
        "dont_pluck_this": "unused",
      })
    ));
    let inv = Invocation::test(file!(), Entity::test("noop"), stream, None)?;
    let mut packets = op
      .handle(
        inv,
        Context::new(config, &InherentData::unsafe_default(), panic_callback()),
      )
      .await?
      .collect::<Vec<_>>()
      .await;
    println!("{:?}", packets);
    let _ = packets.pop().unwrap()?;
    let packet = packets.pop().unwrap()?;
    assert_eq!(packet.decode::<String>()?, "hello");

    Ok(())
  }

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let op = Op::new();
    let config = HashMap::from([("path".to_owned(), json!(["pluck_this"]))]);
    let config = Op::decode_config(Some(config.into()))?;

    let stream = packet_stream!((
      "input",
      serde_json::json!({
        "pluck_this": "hello",
        "dont_pluck_this": "unused",
      })
    ));
    let inv = Invocation::test(file!(), Entity::test("noop"), stream, None)?;
    let mut packets = op
      .handle(
        inv,
        Context::new(config, &InherentData::unsafe_default(), panic_callback()),
      )
      .await?
      .collect::<Vec<_>>()
      .await;
    println!("{:?}", packets);
    let _ = packets.pop().unwrap()?;
    let packet = packets.pop().unwrap()?;
    assert_eq!(packet.decode::<String>()?, "hello");

    Ok(())
  }

  #[tokio::test]
  async fn test_pluck_fn() -> Result<()> {
    let json = serde_json::json!({
      "first": {
        "second": {
          "third" : [
            {"fourth": "first element"},
            {"fourth": "second element"}
          ]
        }
      }
    });

    let val = pluck(
      &json,
      &[
        "first".to_owned(),
        "second".to_owned(),
        "third".to_owned(),
        "0".to_owned(),
        "fourth".to_owned(),
      ],
    );
    assert_eq!(val, Some(&serde_json::json!("first element")));

    Ok(())
  }
}
