use anyhow::Result;
mod utils;
use utils::*;
use wick_packet::{packet_stream, Packet};

#[test_logger::test(tokio::test)]
#[ignore = "TODO:FIX_HUNG"]
async fn panics() -> Result<()> {
  tester(
    "./manifests/v0/errors/panics.yaml",
    packet_stream!(("input", "input")),
    "panics",
    vec![Packet::err("output", "Wat")],
  )
  .await
}

#[test_logger::test(tokio::test)]
#[ignore = "TODO:FIX_HUNG"]
async fn errors() -> Result<()> {
  tester(
    "./manifests/v0/errors/errors.yaml",
    packet_stream!(("input", "input")),
    "errors",
    vec![Packet::err("output", "Wat")],
  )
  .await
}