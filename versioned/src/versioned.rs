pub trait Versioned {
    const VER: u16;
    type Base;
}

// This is a placeholder, that we will use to organize the different versions.
// Because this is a zero-variant enum, it can't actually be instantiated.
enum MyStruct {}

#[derive(Debug, Clone)]
struct MyStructV1 {
    x: u64,
}

// This can be done with a proc macro, e.g. #[derive(Versioned)]
//
// To set the version, the macro could either looks for "VNNN"
// at the end of the name, or accept a version parameter.
//
// To set the base struct, the macro could either use the name
// (with the "VNNN" suffix stripped off), or accept a base
// parameter.
//
// Or maybe that's over-engineered. It's not that hard to write
// this impl by hand, so maybe the proc-macro should only handle
// the automatic case.
impl Versioned for MyStructV1 {
    const VER: u16 = 1;
    type Base = MyStruct;
}

#[derive(Debug, Clone)]
struct MyStructV2 {
    x: u64,
    y: u8,
}

impl Versioned for MyStructV2 {
    const VER: u16 = 2;
    type Base = MyStruct;
}

#[derive(Debug, Clone, PartialEq)]
struct MyStructV3 {
    x: u64,
    y: u64,
}

impl Versioned for MyStructV3 {
    const VER: u16 = 3;
    type Base = MyStruct;
}

pub trait FromVersion<T>: Versioned
where T: Versioned {
    fn from_version(t: T) -> Self;
}

// Like std::convert::From, FromVersion should be reflexive.
// This allows it to be used in generic parameters where any
// version should be accepted.
impl<T> FromVersion<T> for T
where T: Versioned
{
    fn from_version(t: T) -> Self {
        t
    }
}

pub trait IntoVersion<T> {
    fn into_version(self) -> T;
}

// Like std::convert::Into, provide a blanket implementation
// so that From<T> for U implies Into<U> for T
//
// Because there is a blanket `impl From<T> for T`, this also
// implies `impl Into<T> for T`.
impl<T, U> IntoVersion<U> for T
where
    T: Versioned,
    U: FromVersion<T>,
{
    fn into_version(self) -> U {
        U::from_version(self)
    }
}

impl FromVersion<MyStructV1> for MyStructV2 {
    fn from_version(msv1: MyStructV1) -> Self {
        Self { x: msv1.x, y: 0 }
    }
}

impl FromVersion<MyStructV2> for MyStructV3 {
    fn from_version(msv2: MyStructV2) -> Self {
        Self {
            x: msv2.x,
            y: msv2.y.into(),
        }
    }
}

// I should also build a proc-macro that can auto-implement
// this function.
impl FromVersion<MyStructV1> for MyStructV3 {
    fn from_version(msv1: MyStructV1) -> Self {
        let msv2 = MyStructV2::from_version(msv1);
        MyStructV3::from_version(msv2)
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
        let y = MyStructV2::from_version(x.clone());
        println!("{:?}", y);
        let z = MyStructV3::from_version(x);
        assert_eq!(z, MyStructV3 { x: 42, y: 0 })
    }
}
