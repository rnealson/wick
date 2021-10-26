// !!START_LINTS
// Vino lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![deny(
  clippy::expect_used,
  clippy::explicit_deref_methods,
  clippy::option_if_let_else,
  clippy::await_holding_lock,
  clippy::cloned_instead_of_copied,
  clippy::explicit_into_iter_loop,
  clippy::flat_map_option,
  clippy::fn_params_excessive_bools,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::large_types_passed_by_value,
  clippy::manual_ok_or,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::must_use_candidate,
  clippy::needless_for_each,
  clippy::needless_pass_by_value,
  clippy::option_option,
  clippy::redundant_else,
  clippy::semicolon_if_nothing_returned,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::unnested_or_patterns,
  clippy::future_not_send,
  clippy::useless_let_if_seq,
  clippy::str_to_string,
  clippy::inherent_to_string,
  clippy::let_and_return,
  clippy::string_to_string,
  clippy::try_err,
  clippy::if_then_some_else_none,
  bad_style,
  clashing_extern_declarations,
  const_err,
  dead_code,
  deprecated,
  explicit_outlives_requirements,
  improper_ctypes,
  invalid_value,
  missing_copy_implementations,
  missing_debug_implementations,
  mutable_transmutes,
  no_mangle_generic_items,
  non_shorthand_field_patterns,
  overflowing_literals,
  path_statements,
  patterns_in_fns_without_body,
  private_in_public,
  trivial_bounds,
  trivial_casts,
  trivial_numeric_casts,
  type_alias_bounds,
  unconditional_recursion,
  unreachable_pub,
  unsafe_code,
  unstable_features,
  // unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
// !!END_LINTS
// Add exceptions here
#![allow(clippy::expect_used, missing_docs)]

#[macro_use]
extern crate tracing;

use std::sync::Arc;

use structopt::StructOpt;
use vino_keyvalue_redis::provider::Provider;
use vino_provider_cli::cli::DefaultCliOptions;

#[derive(Debug, Clone, StructOpt)]
pub struct Options {
  /// IP address to bind to.
  #[structopt(short = "u", long = "redis-url", env = vino_keyvalue_redis::REDIS_URL_ENV)]
  pub url: String,

  #[structopt(flatten)]
  pub options: DefaultCliOptions,
}

#[tokio::main]
async fn main() -> Result<(), vino_keyvalue_redis::error::Error> {
  let opts = Options::from_args();
  let url = opts.url;
  let _guard = vino_provider_cli::init_logging(&opts.options.logging.name("keyvalue-redis"));
  let provider = Provider::default();
  provider.connect("default".to_owned(), url.clone()).await?;

  vino_provider_cli::init_cli(Arc::new(provider), Some(opts.options.into())).await?;
  Ok(())
}
