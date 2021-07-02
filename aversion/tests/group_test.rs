use aversion::group::{DataSink, GroupHeader, UpgradeLatest};
use aversion::util::cbor::CborData;
use aversion::{
    assign_message_ids, FromVersion, GroupDeserialize, MessageId, UpgradeLatest, Versioned,
};
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Seek, SeekFrom};

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

#[derive(Debug, PartialEq, GroupDeserialize)]
enum MyGroup1 {
    Foo(Foo),
    Bar(Bar),
}

assign_message_ids! {
    Foo: 123,
    Bar: 999,
}

#[test]
fn test_group() {
    let cursor = Cursor::new(Vec::<u8>::new());
    let mut out_stream = CborData::new(cursor);

    let my_foo = FooV1 { foo: 1234 };
    out_stream.write_message(&my_foo).unwrap();
    let cursor = out_stream.into_inner();
    {
        let mut cursor = cursor.clone();
        // Reset the cursor so we will read from the beginning.
        cursor.seek(SeekFrom::Start(0)).unwrap();

        let mut my_stream = CborData::new(cursor);

        let message = MyGroup1::read_message(&mut my_stream).unwrap();
        assert_eq!(message, MyGroup1::Foo(Foo { foo3: 1245 }));
    }
    {
        let mut cursor = cursor.clone();
        // Reset the cursor so we will read from the beginning.
        cursor.seek(SeekFrom::Start(0)).unwrap();

        let mut my_stream = CborData::new(cursor);

        let message: Foo = MyGroup1::expect_message(&mut my_stream).unwrap();
        assert_eq!(message, Foo { foo3: 1245 });
    }
}
