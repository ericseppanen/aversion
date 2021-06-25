pub mod group;
mod id;
mod versioned;

#[doc(inline)]
pub use crate::versioned::{FromVersion, IntoVersion, Versioned};

#[doc(inline)]
pub use versioned_macros::{UpgradeLatest, Versioned};

#[doc(inline)]
pub use id::MessageId;
