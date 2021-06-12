trait Versioned {
    const VER: u16;
    const MSGID: u16;
    type Base;
}

type MyStruct = MyStructV1;

#[derive(Debug, Clone)]
struct MyStructV1 {
    x: u64,
}

const MYSTRUCT_MSGID: u16 = 1234;

// This can be done with a proc macro.
// It either looks for "VNNN" at the end of the name,
// or you need to pass the version.
//
// So we would need:  #[derive(Versioned(1234, 1))]
//
impl Versioned for MyStructV1 {
    const VER: u16 = 1;
    const MSGID: u16 = MYSTRUCT_MSGID;
    type Base = MyStruct;
}

#[derive(Debug, Clone)]
struct MyStructV2 {
    x: u64,
    y: u8,
}

impl Versioned for MyStructV2 {
    const VER: u16 = 2;
    const MSGID: u16 = MyStructV1::MSGID;
    type Base = MyStruct;
}

#[derive(Debug, Clone, PartialEq)]
struct MyStructV3 {
    x: u64,
    y: u64,
}

impl Versioned for MyStructV3 {
    const VER: u16 = 3;
    const MSGID: u16 = MyStructV1::MSGID;
    type Base = MyStruct;
}

// TODO: write a trait VersionInto ?

trait VersionFrom<T> {
    fn vfrom(t: T) -> Self;
}

impl VersionFrom<MyStructV1> for MyStructV2 {
    fn vfrom(msv1: MyStructV1) -> Self {
        Self { x: msv1.x, y: 0 }
    }
}

impl VersionFrom<MyStructV2> for MyStructV3 {
    fn vfrom(msv2: MyStructV2) -> Self {
        Self {
            x: msv2.x,
            y: msv2.y.into(),
        }
    }
}

// I should also build a proc-macro that can auto-implement
// this function.
impl VersionFrom<MyStructV1> for MyStructV3 {
    fn vfrom(msv1: MyStructV1) -> Self {
        let msv2 = MyStructV2::vfrom(msv1);
        MyStructV3::vfrom(msv2)
    }
}

// TODO:
// In order to actually use any of these VersionFrom impls,
// I need a network messenger object that holds a registry of
// (message_id, version) -> Fn
//
// Then I need a macro that can easily plug in an adapter Fn for
// each of the types.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter() {
        let x = MyStructV1 { x: 42 };
        let y = MyStructV2::vfrom(x.clone());
        println!("{:?}", y);
        let z = MyStructV3::vfrom(x);
        assert_eq!(z, MyStructV3 { x: 42, y: 0 })
    }
}
