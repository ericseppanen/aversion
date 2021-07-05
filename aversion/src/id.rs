use crate::Versioned;

/// Trait for data structures with a message type number.
///
/// Each data structure that has a message type may be deserialized
/// from a context-free buffer (e.g. a file or network socket).
pub trait MessageId: Versioned {
    /// The message id.
    ///
    /// This is a constant that identifies this message when it is
    /// serialized. The same id will be used for all versions of
    /// this message type.
    const MSG_ID: u16;
}
