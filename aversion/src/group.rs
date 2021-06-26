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
/// This trait will normally be derived by a macro.
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
    type Error;
    type Header: GroupHeader;

    fn read_header(&mut self) -> Result<Self::Header, Self::Error>;
    fn read_message<T>(&mut self) -> Result<T, Self::Error>
    where
        T: DeserializeOwned;

    /// An unknown message id was received.
    fn unknown_message(&self, msg_id: u16) -> Self::Error {
        panic!("unknown message id {}", msg_id);
    }

    /// An unknown version of a known message was received.
    fn unknown_version<T>(&self, ver: u16) -> Self::Error {
        panic!("unknown version {} for {}", ver, type_name::<T>());
    }

    /// Expected a specific message type, but got a different message id.
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
    fn read_message<Src>(src: &mut Src) -> Result<Self, Src::Error>
    where
        Src: DataSource;

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
