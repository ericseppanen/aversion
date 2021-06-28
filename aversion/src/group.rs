//! Define message groups for automatic dispatching.
//!
//! A message group is a collection of messages that may be used together.
//! For example, a file format or a network protocol may form a group.
//!

use std::any::type_name;

use crate::{MessageId, Versioned};
use serde::de::DeserializeOwned;

/// A data structure that contains a message-id and version fields.

pub trait GroupHeader {
    fn msg_id(&self) -> u16;
    fn msg_ver(&self) -> u16;
}

/// A trait for deserializing any version of a [`Versioned`] data structure.
///
/// This trait will normally be derived using `#[derive(Versioned)]`.
///
// How will the macro know which versions exist?
// a) Macro will assume that every version [1..latest] exists
//    - and maybe there's a macro to generate stubs for missing versions?
// b) User needs to specify a range or list of versions
pub trait UpgradeLatest: DeserializeOwned + Versioned {
    /// Deserialize version `ver` of the target struct, then upgrade it to the latest version.
    fn upgrade_latest<Src>(src: &mut Src, ver: u16) -> Result<Self, Src::Error>
    where
        Src: DataSource;
}

/// `DataSource` allows user-defined IO, deserialization, and
/// error handling.
///
pub trait DataSource {
    /// A user-defined error type.
    ///
    /// This error type will be returned from [`read_header`][Self::read_header]
    /// and [`read_message`][Self::read_message].
    /// It's probably a good idea for it to be able to represent IO errors,
    /// deserialization errors, and "unknown message" errors.
    type Error;
    /// A user-defined header struct.
    ///
    /// The `Header` is a way of communicating what kind of message is being
    /// sent, along with the message version.
    type Header: GroupHeader;

    /// Read a header from the data source.
    ///
    /// This is a user-defined function that will read the next header.
    /// The data in the header will be used to determine what kind of
    /// message comes next.
    ///
    fn read_header(&mut self) -> Result<Self::Header, Self::Error>;

    /// Read a message from the data source.
    ///
    /// This is a user-defined function that will deserialize a message
    /// of type `T`.
    fn read_message<T>(&mut self) -> Result<T, Self::Error>
    where
        T: DeserializeOwned;

    /// An unknown message id was received.
    ///
    /// This is a user-defined function that constructs an error value.
    /// This function will be called by [`GroupDeserialize::read_message`]
    /// when an unknown message is received (a message with an unknown
    /// message id).
    ///
    fn unknown_message(&self, msg_id: u16) -> Self::Error {
        panic!("unknown message id {}", msg_id);
    }

    /// An unknown version of a known message was received.
    ///
    /// This is a user-defined function that constructs an error value.
    /// This function will be called by [`GroupDeserialize::read_message`]
    /// when a known message id is received, but with a message version that
    /// is unknown.
    ///
    fn unknown_version<T>(&self, ver: u16) -> Self::Error {
        panic!("unknown version {} for {}", ver, type_name::<T>());
    }

    /// Expected a specific message type, but got a different message id.
    ///
    /// This is a user-defined function that constructs an error value.
    /// This function will be called by [`GroupDeserialize::expect_message`]
    /// when a different message id is received from the message that was
    /// specified.
    ///
    fn unexpected_message<T>(&self, msg_id: u16) -> Self::Error {
        panic!(
            "unexpected message id {} (expected {})",
            msg_id,
            type_name::<T>()
        );
    }
}

/// A derived trait that can deserialize any message from a group.
pub trait GroupDeserialize: Sized {
    /// Read the next message from the `DataSource`.
    ///
    /// This will read the message header, and if the message id and
    /// message version are known, also read the message.
    /// The message will be upgraded to the latest version, and then
    /// returned as an enum variant (in the `Self` enum).
    fn read_message<Src>(src: &mut Src) -> Result<Self, Src::Error>
    where
        Src: DataSource;

    /// Read a specific message type from the `DataSource`.
    ///
    /// This will read the message header, and if the message id matches
    /// the type `T` that was requested, read the message.
    /// The message will be upgraded to the latest version, and then
    /// returned.
    fn expect_message<Src, T>(src: &mut Src) -> Result<T, Src::Error>
    where
        Src: DataSource,
        T: MessageId + UpgradeLatest,
    {
        let header: Src::Header = src.read_header()?;
        if header.msg_id() == T::MSG_ID {
            T::upgrade_latest(src, header.msg_ver())
        } else {
            // Call the user-supplied error fn
            Err(src.unexpected_message::<T>(header.msg_id()))
        }
    }
}
