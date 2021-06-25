use byteorder::{BigEndian, ReadBytesExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::Read;
use versioned::group::{GroupDeserialize, GroupHelper};
use versioned::{FromVersion, MessageId, Versioned};

/// A header that can be serialized into a fixed-size buffer.
#[derive(Debug, Clone)]
pub struct BasicFixedHeader {
    pub msg_id: u16,
    pub msg_ver: u16,
    pub length: u32,
}

impl BasicFixedHeader {
    /// Deserialize a header.
    // FIXME: returning Option is weird.
    // FIXME: How should this look for user-defined headers?
    // what kind of error should it return?
    pub fn deserialize_from(r: &mut impl Read) -> Result<Self, MyGroupError> {
        let msg_id = r.read_u16::<BigEndian>()?;
        let msg_ver = r.read_u16::<BigEndian>()?;
        let length = r.read_u32::<BigEndian>()?;
        Ok(BasicFixedHeader {
            msg_id,
            msg_ver,
            length,
        })
    }
}
enum FooBase {}

#[derive(Versioned, Serialize, Deserialize)]
struct FooV1 {
    foo: u32,
}

type Foo = FooV1;

enum BarBase {}

#[derive(Versioned)]
struct BarV1 {
    bar: u64,
    bar2: bool,
}

type Bar = BarV1;

// This should be derived
enum MyGroup1 {
    Foo(Foo),
    Bar(Bar),
}

// This should be derived
impl MessageId for FooV1 {
    const MSG_ID: u16 = 0x70;
}

// This should be derived
impl MessageId for BarV1 {
    const MSG_ID: u16 = 0x71;
}

pub struct MyGroupError;

impl From<serde_cbor::Error> for MyGroupError {
    fn from(_: serde_cbor::Error) -> Self {
        MyGroupError
    }
}

impl From<std::io::Error> for MyGroupError {
    fn from(_: std::io::Error) -> Self {
        MyGroupError
    }
}

struct MyStream {
    reader: Box<dyn Read>,
}

impl GroupHelper for MyStream {
    type Error = MyGroupError;
    type Header = BasicFixedHeader;

    fn read_header<T>(&mut self) -> Result<T, Self::Error> {
        todo!()
    }

    fn read_message<T>(&mut self) -> Result<T, Self::Error>
    where
        T: DeserializeOwned,
    {
        let msg: T = serde_cbor::from_reader(&mut self.reader)?;
        Ok(msg)
    }
}

impl GroupDeserialize for MyGroup1 {
    type Source = MyStream;

    // Assume that the macro has some input like
    // Deserializer = serde_cbor::from_reader so that
    // the macro can paste that in.

    fn read_message(src: &mut Self::Source) -> Result<Self, <Self::Source as GroupHelper>::Error> {
        let header: <Self::Source as GroupHelper>::Header = src.read_header()?;
        match (header.msg_id, header.msg_ver) {
            (Foo::MSG_ID, 1) => {
                let msg = src.read_message::<FooV1>()?;
                let upgraded = Foo::from_version(msg);
                Ok(MyGroup1::Foo(upgraded))
            }
            _ => {
                // Call the user-supplied error fn
                Err(src.unknown_message(header))
            }
        }
    }
    fn expect_message<T>(src: &mut Self::Source) -> Result<T, <Self::Source as GroupHelper>::Error>
    where
        T: MessageId,
    {
        let header: <Self::Source as GroupHelper>::Header = src.read_header()?;
        if header.msg_id == T::MSG_ID {
            // Need a trait for "deserialize any version of struct T"
            todo!()
        }
        // Call src.unknown_message()? It really isn't unknown, just unexpected...
        todo!()
    }
}

#[test]
fn test_group() {}
