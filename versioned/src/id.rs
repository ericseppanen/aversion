use crate::Versioned;
use serde::{Deserialize, Serialize};

/// Trait for data structures with a message type number.
///
/// Each data structure that has a message type may be deserialized
/// from a context-free buffer (e.g. a file or network socket).
pub trait MessageId: Versioned {
    const MSG_ID: u16;
}

// FIXME: maybe this should be a trait, so that different
// clients can define their own message header.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageHeader {
    id: u16,
    ver: u16,
    len: u32,
}
