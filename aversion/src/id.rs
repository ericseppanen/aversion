use crate::Versioned;

/// Trait for data structures with a message type number.
///
/// Each data structure that has a message type may be deserialized
/// from a context-free buffer (e.g. a file or network socket).
pub trait MessageId: Versioned {
    const MSG_ID: u16;
}
