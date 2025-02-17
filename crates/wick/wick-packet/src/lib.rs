#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/104781277?s=96&v=4")]
#![doc = include_str!("../README.md")]
// !!START_LINTS
// Wick lints
// Do not change anything between the START_LINTS and END_LINTS line.
// This is automatically generated. Add exceptions after this section.
#![allow(unknown_lints)]
#![deny(
  clippy::await_holding_lock,
  clippy::borrow_as_ptr,
  clippy::branches_sharing_code,
  clippy::cast_lossless,
  clippy::clippy::collection_is_never_read,
  clippy::cloned_instead_of_copied,
  clippy::cognitive_complexity,
  clippy::create_dir,
  clippy::deref_by_slicing,
  clippy::derivable_impls,
  clippy::derive_partial_eq_without_eq,
  clippy::equatable_if_let,
  clippy::exhaustive_structs,
  clippy::expect_used,
  clippy::expl_impl_clone_on_copy,
  clippy::explicit_deref_methods,
  clippy::explicit_into_iter_loop,
  clippy::explicit_iter_loop,
  clippy::filetype_is_file,
  clippy::flat_map_option,
  clippy::format_push_string,
  clippy::fn_params_excessive_bools,
  clippy::future_not_send,
  clippy::get_unwrap,
  clippy::implicit_clone,
  clippy::if_then_some_else_none,
  clippy::impl_trait_in_params,
  clippy::implicit_clone,
  clippy::inefficient_to_string,
  clippy::inherent_to_string,
  clippy::iter_not_returning_iterator,
  clippy::large_types_passed_by_value,
  clippy::large_include_file,
  clippy::let_and_return,
  clippy::manual_assert,
  clippy::manual_ok_or,
  clippy::manual_split_once,
  clippy::manual_let_else,
  clippy::manual_string_new,
  clippy::map_flatten,
  clippy::map_unwrap_or,
  clippy::missing_enforced_import_renames,
  clippy::missing_assert_message,
  clippy::missing_const_for_fn,
  clippy::must_use_candidate,
  clippy::mut_mut,
  clippy::needless_for_each,
  clippy::needless_option_as_deref,
  clippy::needless_pass_by_value,
  clippy::needless_collect,
  clippy::needless_continue,
  clippy::non_send_fields_in_send_ty,
  clippy::nonstandard_macro_braces,
  clippy::option_if_let_else,
  clippy::option_option,
  clippy::rc_mutex,
  clippy::redundant_else,
  clippy::same_name_method,
  clippy::semicolon_if_nothing_returned,
  clippy::str_to_string,
  clippy::string_to_string,
  clippy::too_many_lines,
  clippy::trivially_copy_pass_by_ref,
  clippy::trivial_regex,
  clippy::try_err,
  clippy::unnested_or_patterns,
  clippy::unused_async,
  clippy::unwrap_or_else_default,
  clippy::useless_let_if_seq,
  bad_style,
  clashing_extern_declarations,
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
  unused,
  unused_allocation,
  unused_comparisons,
  unused_import_braces,
  unused_parens,
  unused_qualifications,
  while_true,
  missing_docs
)]
#![warn(clippy::exhaustive_enums)]
#![allow(unused_attributes, clippy::derive_partial_eq_without_eq, clippy::box_default)]
// !!END_LINTS
// Add exceptions here
#![allow(missing_docs)]

mod b64_bytes;
mod collection_link;
mod context;
#[cfg(feature = "datetime")]
mod datetime;
mod entity;
mod error;
mod inherent;
#[cfg(feature = "invocation")]
mod invocation;
mod macros;
mod metadata;
mod output;
mod packet;
mod packet_stream;
mod stream_map;
mod wrapped_type;

#[cfg(feature = "validation")]
pub mod validation;

pub use collection_link::ComponentReference;
pub use context::{ContextTransport, InvocationRequest, RuntimeConfig};
#[cfg(feature = "datetime")]
pub use datetime::{date_from_millis, parse_date, serde, DateTime};
pub use entity::Entity;
pub use error::{Error, ParseError};
pub use inherent::InherentData;
#[cfg(feature = "invocation")]
pub use invocation::{Invocation, InvocationData};
pub use metadata::{Flags, WickMetadata, CLOSE_BRACKET, DONE_FLAG, OPEN_BRACKET};
pub use output::{OutgoingPort, OutputIterator, Port, ValuePort};
pub use packet::{from_raw_wasmrs, from_wasmrs, packetstream_to_wasmrs, Packet, PacketError, PacketPayload};
pub use packet_stream::{into_packet, PacketSender, PacketStream};
pub use stream_map::StreamMap;
pub use wasmrs::Metadata;
pub use wasmrs_rx;
pub use wasmrs_rx::{Flux, FluxChannel, FluxReceiver, Mono, Observable, Observer};
pub use wrapped_type::TypeWrapper;

#[cfg(feature = "rt-tokio")]
mod runtime;
pub use b64_bytes::Base64Bytes;
#[cfg(feature = "rt-tokio")]
pub use runtime::split_stream;

pub(crate) type Result<T> = std::result::Result<T, Error>;
