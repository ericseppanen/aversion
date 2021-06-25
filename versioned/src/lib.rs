mod versioned;
mod id;
pub mod group;

#[doc(inline)]
pub use crate::versioned::{Versioned, FromVersion, IntoVersion};

#[doc(inline)]
pub use versioned_macros::Versioned;

#[doc(inline)]
pub use id::MessageId;
