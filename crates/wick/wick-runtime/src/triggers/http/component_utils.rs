use std::collections::HashMap;

use hyper::header::CONTENT_LENGTH;
use hyper::http::response::Builder;
use hyper::http::{HeaderName, HeaderValue};
use hyper::{Body, Response, StatusCode};
use serde_json::{Map, Value};
use tokio_stream::StreamExt;
use tracing::Span;
use uuid::Uuid;
use wick_config::config::Codec;
use wick_interface_http::types as wick_http;
use wick_packet::{
  packets,
  Base64Bytes,
  Entity,
  InherentData,
  Invocation,
  Packet,
  PacketPayload,
  PacketStream,
  RuntimeConfig,
};

use super::conversions::convert_response;
use super::HttpError;
use crate::Runtime;

pub(super) async fn handle_request_middleware(
  tx_id: Uuid,
  target: Entity,
  operation_config: Option<RuntimeConfig>,
  runtime: Runtime,
  req: &wick_http::HttpRequest,
  span: &Span,
) -> Result<Option<wick_http::RequestMiddlewareResponse>, HttpError> {
  let packets = packets!(("request", req));
  let invocation = Invocation::new_with_id(
    tx_id,
    Entity::server("http_client"),
    target.clone(),
    packets,
    InherentData::unsafe_default(),
    span,
  );

  let stream = runtime
    .invoke(invocation, operation_config)
    .await
    .map_err(|e| HttpError::OperationError(e.to_string()))?;

  let packets = stream.collect::<Result<Vec<Packet>, _>>().await;

  let packets = match packets {
    Ok(packets) => packets,
    Err(e) => return Err(HttpError::InvalidPreRequestResponse(e.to_string())),
  };

  if let Some(packet) = packets.iter().find(|p| p.is_error()) {
    return Err(HttpError::InvalidPreRequestResponse(
      packet.clone().unwrap_err().msg().to_owned(),
    ));
  };

  let Some(packet) = packets.into_iter().find(|p| p.has_data()) else {
    return Err(HttpError::PreRequestResponseNoData(target));
  };

  if packet.port() == "output" {
    let response: wick_http::RequestMiddlewareResponse = packet
      .decode()
      .map_err(|e| HttpError::InvalidPreRequestResponse(e.to_string()))?;

    Ok(Some(response))
  } else {
    Err(HttpError::InvalidPreRequestResponse(format!(
      "Invalid response named {}, pre-request middleware expects a response named 'output' that is either an HttpRequest or an HttpResponse",
      packet.port()
    )))
  }
}

pub(super) async fn handle_response_middleware(
  tx_id: Uuid,
  target: Entity,
  operation_config: Option<RuntimeConfig>,
  runtime: Runtime,
  req: &wick_http::HttpRequest,
  res: &wick_http::HttpResponse,
  span: &Span,
) -> Result<Option<wick_http::HttpResponse>, HttpError> {
  let packets = packets!(("request", req), ("response", res));
  let invocation = Invocation::new_with_id(
    tx_id,
    Entity::server("http_client"),
    target.clone(),
    packets,
    InherentData::unsafe_default(),
    span,
  );

  let stream = runtime
    .invoke(invocation, operation_config)
    .await
    .map_err(|e| HttpError::OperationError(e.to_string()))?;

  let packets = stream.collect::<Result<Vec<Packet>, _>>().await;

  let packets = match packets {
    Ok(packets) => packets,
    Err(e) => return Err(HttpError::InvalidPostRequestResponse(e.to_string())),
  };

  if let Some(packet) = packets.iter().find(|p| p.is_error()) {
    return Err(HttpError::InvalidPostRequestResponse(
      packet.clone().unwrap_err().msg().to_owned(),
    ));
  };

  let Some(packet) = packets.into_iter().find(|p| p.has_data()) else {
    return Err(HttpError::PostRequestResponseNoData(target));
  };

  if packet.port() == "response" {
    let response: wick_http::HttpResponse = packet
      .decode()
      .map_err(|e| HttpError::InvalidPostRequestResponse(e.to_string()))?;

    Ok(Some(response))
  } else {
    Err(HttpError::InvalidPostRequestResponse("Invalid packet".to_owned()))
  }
}

pub(super) async fn respond(
  codec: Codec,
  stream: Result<PacketStream, HttpError>,
) -> Result<Response<Body>, HttpError> {
  if let Err(e) = stream {
    return Ok(
      Builder::new()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(e.to_string()))
        .unwrap(),
    );
  }
  let mut stream = stream.unwrap();
  let mut builder = Response::builder();
  let mut body = bytes::BytesMut::new();
  while let Some(packet) = stream.next().await {
    match packet {
      Ok(p) => {
        if p.port() == "response" {
          if let PacketPayload::Err(e) = p.payload() {
            return Err(HttpError::OutputStream(p.port().to_owned(), e.msg().to_owned()));
          }
          if p.is_done() {
            continue;
          }
          let response: wick_interface_http::types::HttpResponse = p
            .decode()
            .map_err(|e| HttpError::Deserialize("response".to_owned(), e.to_string()))?;
          builder = convert_response(builder, response)?;
        } else if p.port() == "body" {
          if let PacketPayload::Err(e) = p.payload() {
            return Err(HttpError::OutputStream(p.port().to_owned(), e.msg().to_owned()));
          }
          if !p.has_data() {
            continue;
          }
          if codec == Codec::Json {
            let response: Value = p.decode().map_err(|e| HttpError::Codec(codec, e.to_string()))?;
            let as_str = response.to_string();
            let bytes = as_str.as_bytes();
            body.extend_from_slice(bytes);
          } else {
            let response: Base64Bytes = p.decode().map_err(|e| HttpError::Bytes(e.to_string()))?;
            body.extend_from_slice(&response);
          }
        }
      }
      Err(e) => return Err(HttpError::OperationError(e.to_string())),
    }
  }
  builder = reset_header(builder, CONTENT_LENGTH, body.len());
  Ok(builder.body(body.freeze().into()).unwrap())
}

fn reset_header(mut builder: Builder, header: HeaderName, value: impl Into<HeaderValue>) -> Builder {
  #[allow(clippy::option_if_let_else)]
  if let Some(headers) = builder.headers_mut() {
    if let Some(cl) = headers.get_mut(&header) {
      *cl = value.into();
    } else {
      headers.insert(header, value.into());
    }
  } else {
    builder = builder.header(header, value.into());
  };
  builder
}

enum MapVal {
  RawVal(Value),
  RootArray(Vec<Value>),
}

pub(super) async fn stream_to_json(stream: PacketStream) -> Result<Value, HttpError> {
  let mut stream = stream;
  let mut map = HashMap::new();
  while let Some(packet) = stream.next().await {
    match packet {
      Ok(p) => {
        if let PacketPayload::Err(err) = p.payload() {
          return Err(HttpError::OutputStream(p.port().to_owned(), err.msg().to_owned()));
        }
        if !p.has_data() {
          continue;
        }
        let port = p.port().to_owned();
        if let Some(val) = map.remove(p.port()) {
          let val = match val {
            MapVal::RawVal(v) => {
              let response: Value = p
                .decode_value()
                .map_err(|e| HttpError::Codec(Codec::Json, e.to_string()))?;
              MapVal::RootArray(vec![v, response])
            }
            MapVal::RootArray(mut v) => {
              let response: Value = p
                .decode_value()
                .map_err(|e| HttpError::Codec(Codec::Json, e.to_string()))?;
              v.push(response);
              MapVal::RootArray(v)
            }
          };
          map.insert(port, val);
        } else {
          let response: Value = p
            .decode_value()
            .map_err(|e| HttpError::Codec(Codec::Json, e.to_string()))?;
          map.insert(port, MapVal::RawVal(response));
        }
      }
      Err(e) => return Err(HttpError::OperationError(e.to_string())),
    }
  }
  let json = Value::Object(
    map
      .into_iter()
      .map(|(k, v)| {
        (
          k,
          match v {
            MapVal::RawVal(v) => v,
            MapVal::RootArray(v) => Value::Array(v),
          },
        )
      })
      .collect::<Map<String, Value>>(),
  );
  Ok(json)
}
