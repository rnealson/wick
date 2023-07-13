use serde_value::Value;
use thiserror::Error;
use wick_packet::Packet;

#[derive(Error, Debug, PartialEq)]
pub enum TestError {
  #[error("Could not read file : {0}")]
  ReadFailed(String),
  #[error("Could not parse contents as YAML : {0}")]
  ParseFailed(String),
  #[error("Invocation failed: {0}")]
  InvocationFailed(String),
  #[error("Invocation timed out: {0}")]
  InvocationTimeout(String),
  #[error("Serialization failed: {0}")]
  Serialization(String),
  #[error("Deserialization failed: {0}")]
  Deserialization(String),
  #[error("Could not render configuration: {0}")]
  Configuration(String),
  #[error("Could not create component instance to test: {0}")]
  Factory(String),
  #[error("Could not find operation {0} on this component")]
  OpNotFound(String),
  #[error(transparent)]
  ConfigUnsatisfied(wick_packet::Error),
  #[error("Test input sent packets after marking input '{0}' as done")]
  PacketsAfterDone(String),
  #[error("Got an output packet for a port '{0}' we've never seen")]
  InvalidPort(String),
  #[error("Assertion failed")]
  Assertion(Packet, Packet, AssertionFailure),
}

#[derive(Error, Debug, PartialEq)]
pub enum AssertionFailure {
  #[error("Payload mismatch")]
  Payload(Value, Value),
  #[error("Expected data in packet but got none")]
  ActualNoData,
  #[error("Expected no data in packet but got some")]
  ExpectedNoData,
  #[error("Flag mismatch")]
  Flags(u8, u8),
  #[error("Port name mismatch")]
  Name(String, String),
}
