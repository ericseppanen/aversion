//! A message group is a collection of messages that may be used together.
//! For example, a file format or a network protocol may form a group.
//!

use crate::MessageId;
use serde::de::DeserializeOwned;

pub trait GroupHelper {
    type Error;
    type Header;

    fn read_header<T>(&mut self) -> Result<T, Self::Error>;
    fn read_message<T>(&mut self) -> Result<T, Self::Error>
    where
        T: DeserializeOwned;

    fn unknown_message(&self, header: Self::Header) -> Self::Error {
        // Suppress "unused variable" warning; this is just a default
        // implementation, but _header would show up in the docs.
        let _ = header;
        panic!("unknown message");
    }
}

pub trait GroupDeserialize: Sized {
    type Source: GroupHelper;

    fn read_message(src: &mut Self::Source) -> Result<Self, <Self::Source as GroupHelper>::Error>;
    fn expect_message<T>(src: &mut Self::Source) -> Result<T, <Self::Source as GroupHelper>::Error>
    where
        T: MessageId;
}
