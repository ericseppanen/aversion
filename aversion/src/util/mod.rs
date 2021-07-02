//! Useful data structures for use with `aversion`
//!
//! This module provides sample data structures for users who
//! just want to get started and don't want to implement the
//! [`DataSource`] trait themselves.
//!
//! [`FixedHeader`] is a basic message header struct that implements
//! the [`GroupHeader`] trait.
//!
//! The [`cbor`] module includes [`CborData`], a `DataSource`/`DataSink`
//! that uses the CBOR serialization format for messages.
//!
//! [`DataSource`]: crate::group::DataSource
//! [`GroupHeader`]: crate::group::GroupHeader
//! [`CborData`]: crate::util::cbor::CborData

mod header;

#[doc(inline)]
pub use header::{BasicHeader, TinyHeader};

#[cfg(feature = "serde_cbor")]
pub mod cbor;
