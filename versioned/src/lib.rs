mod versioned;
mod id;
pub mod group;

#[doc(inline)]
pub use versioned::{Versioned, FromVersion, IntoVersion};

#[doc(inline)]
pub use versioned_macros::Versioned;

#[doc(inline)]
pub use id::MessageId;
