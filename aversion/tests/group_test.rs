use aversion::group::{DataSource, GroupHeader, UpgradeLatest};
use aversion::{
    assign_message_ids, FromVersion, GroupDeserialize, MessageId, UpgradeLatest, Versioned,
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

/// A header that can be serialized into a fixed-size buffer.
#[derive(Debug, Clone)]
pub struct BasicFixedHeader {
    pub msg_id: u16,
    pub msg_ver: u16,
}

impl BasicFixedHeader {
    pub fn for_msg<T>(_msg: &T) -> Self
    where
        T: Versioned,
        T::Base: MessageId,
    {
        BasicFixedHeader {
            msg_id: T::Base::MSG_ID,
            msg_ver: T::VER,
        }
    }

    pub fn new(msg_id: u16, msg_ver: u16) -> Self {
        BasicFixedHeader { msg_id, msg_ver }
    }

    /// Deserialize a header from a `Read` stream.
    pub fn deserialize_from(r: &mut impl Read) -> Result<Self, MyGroupError> {
        let msg_id = r.read_u16::<BigEndian>()?;
        let msg_ver = r.read_u16::<BigEndian>()?;
        Ok(BasicFixedHeader { msg_id, msg_ver })
    }

    /// Serialize a header into a `Write` stream.
    pub fn serialize_into(&self, w: &mut impl Write) -> Result<(), MyGroupError> {
        w.write_u16::<BigEndian>(self.msg_id)?;
        w.write_u16::<BigEndian>(self.msg_ver)?;
        Ok(())
    }
}

// Maybe this should be derived?
impl GroupHeader for BasicFixedHeader {
    fn msg_id(&self) -> u16 {
        self.msg_id
    }

    fn msg_ver(&self) -> u16 {
        self.msg_ver
    }
}

#[derive(Debug, PartialEq, Versioned, Serialize, Deserialize)]
struct FooV1 {
    foo: u32,
}

#[derive(Debug, PartialEq, Versioned, Serialize, Deserialize)]
struct FooV2 {
    foo2: u32,
}

impl FromVersion<FooV1> for FooV2 {
    fn from_version(v1: FooV1) -> Self {
        Self { foo2: v1.foo + 1 }
    }
}

#[derive(Debug, PartialEq, Versioned, Serialize, Deserialize, UpgradeLatest)]
struct FooV3 {
    foo3: u32,
}

impl FromVersion<FooV2> for FooV3 {
    fn from_version(v2: FooV2) -> Self {
        Self { foo3: v2.foo2 + 10 }
    }
}

/// This is the latest version.
type Foo = FooV3;

#[derive(Debug, PartialEq, Versioned, Serialize, Deserialize, UpgradeLatest)]
struct BarV1 {
    bar: u64,
}

/// This is the latest version.
type Bar = BarV1;

#[derive(Debug, PartialEq)]
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

// This impl is user-defined.
impl DataSource for MyStream {
    type Error = MyGroupError;
    type Header = BasicFixedHeader;

    fn read_header(&mut self) -> Result<Self::Header, Self::Error> {
        BasicFixedHeader::deserialize_from(&mut self.reader)
    }

    fn read_message<T>(&mut self) -> Result<T, Self::Error>
    where
        T: DeserializeOwned,
    {
        let msg: T = serde_cbor::from_reader(&mut self.reader)?;
        Ok(msg)
    }
}

#[derive(Debug, PartialEq, GroupDeserialize)]
enum MyGroup1 {
    Foo(Foo),
    Bar(Bar),
}

assign_message_ids! {
    Foo: 123,
    Bar: 999
}

#[test]
fn test_group() {
    let mut cursor = Cursor::new(Vec::<u8>::new());

    let my_foo = FooV1 { foo: 1234 };
    let header = BasicFixedHeader::for_msg(&my_foo);

    // FIXME: add a DataSink trait for writing
    header.serialize_into(&mut cursor).unwrap();
    serde_cbor::to_writer(&mut cursor, &my_foo).unwrap();

    {
        let mut cursor = cursor.clone();
        // Reset the cursor so we will read from the beginning.
        cursor.seek(SeekFrom::Start(0)).unwrap();

        let mut my_stream = MyStream {
            reader: Box::new(cursor),
        };

        let message = MyGroup1::read_message(&mut my_stream).unwrap();
        assert_eq!(message, MyGroup1::Foo(Foo { foo3: 1245 }));
    }
    {
        let mut cursor = cursor.clone();
        // Reset the cursor so we will read from the beginning.
        cursor.seek(SeekFrom::Start(0)).unwrap();

        let mut my_stream = MyStream {
            reader: Box::new(cursor),
        };

        let message: Foo = MyGroup1::expect_message(&mut my_stream).unwrap();
        assert_eq!(message, Foo { foo3: 1245 });
    }
}
