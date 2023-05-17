mod import_types;
mod test1 {
  use crate::import_types::*;
  # [cfg_attr (target_family = "wasm" , async_trait :: async_trait (? Send))]
  #[cfg_attr(not(target_family = "wasm"), async_trait::async_trait)]
  impl OpTestop for Component {
    #[allow(unused)]
    async fn testop(
      message: WickStream<types::http::HttpResponse>,
      outputs: OpTestopOutputs,
      ctx: Context<OpTestopConfig>,
    ) -> Result<()> {
      Ok(())
    }
  }
  #[cfg(test)]
  mod test {

    use anyhow::Result;
    use flow_component::{panic_callback, Context};
    use types::http;
    use wasmrs_guest::{FluxChannel, StreamExt};
    use wick_packet::ContextTransport;

    use super::*;
    use crate::import_types::types::http::HttpResponse;

    #[tokio::test]
    async fn test_typegen() -> Result<()> {
      // Don't delete, it tests that local structs are genned correctly.
      let _local_type = types::LocalStruct {
        field1: "value".to_owned(),
        inner: types::LocalStructInner {
          field1: "value".to_owned(),
          field2: "value".to_owned(),
        },
      };

      let response = HttpResponse {
        version: http::HttpVersion::Http11,
        status: http::StatusCode::Ok,
        headers: Default::default(),
        body: Default::default(),
      };
      let packets = tokio_stream::iter(vec![Ok(response)]).boxed();
      let tx = FluxChannel::new();
      let outputs = OpTestopOutputs::new(tx);
      let ctx = Context::new(OpTestopConfig::default(), None, panic_callback());

      Component::testop(packets, outputs, ctx).await?;
      Ok(())
    }

    #[tokio::test]
    async fn test_configgen() -> Result<()> {
      // Don't delete, it tests that local structs are genned correctly.
      let config = OpTestopConfig {
        a: "value".to_owned(),
        b: 2,
      };
      let expected = ContextTransport::new(config, None);
      let bytes = wasmrs_codec::messagepack::serialize(&expected).unwrap();
      let actual: ContextTransport<OpTestopConfig> = wasmrs_codec::messagepack::deserialize(&bytes).unwrap();
      assert_eq!(actual, expected);

      Ok(())
    }
  }
}