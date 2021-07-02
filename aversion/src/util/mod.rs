//! Useful data structures for use with `aversion`
//!
//! This module provides sample data structures for users who
//! just want to get started and don't want to implement the
//! [`DataSource`] trait themselves.
//!
//! [`FixedHeader`] is a basic message header struct that implements
//! the [`GroupHeader`] trait.
//!
//! The [`cbor`] module includes [`CborSource`], a `DataSource` that
//! uses the CBOR serialization format for messages.
//!
//! [`DataSource`]: crate::group::DataSource
//! [`GroupHeader`]: crate::group::GroupHeader
//! [`CborSource`]: crate::util::cbor::CborSource

mod fixed_header;

#[doc(inline)]
pub use fixed_header::FixedHeader;

#[cfg(feature = "serde_cbor")]
pub mod cbor;
