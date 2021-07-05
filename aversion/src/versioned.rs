/// A data structure that has a version number.
///
/// A `Versioned` data structure is part of a family of data structures,
/// with the expectation that each new version can be deterministically
/// created from the older version using the [`FromVersion`] or
/// [`IntoVersion`] traits.
///
pub trait Versioned {
    /// The data structure version.
    ///
    /// The version field should start at 1, and be incremented each time
    /// a change is made to the data structure. [`FromVersion`] should be
    /// implemented for each new version, to allow automatic upgrading
    /// from the previous version.
    ///
    /// The [`UpgradeLatest`] trait can be derived, to automatically
    /// upgrade from any old version to the latest version.
    const VER: u16;
    /// The data structure base type.
    ///
    /// The `Base` type is the latest version of the data structure.
    /// The `Base` type is the one that will implement [`UpgradeLatest`]
    /// and [`MessageId`]. Those two traits should not be needed on older
    /// versions.
    type Base: Versioned;
}

/// Convert an older message version to a newer message version.
///
/// Like the standard library's `From` and `Into` traits, users should
/// implement `FromVersion`, and the corresponding [`IntoVersion`]
/// implementation is provided by a blanket implementation.
///
pub trait FromVersion<T>: Versioned
where
    T: Versioned,
{
    /// Convert from an older `Versioned` type to a newer `Versioned` type.
    fn from_version(t: T) -> Self;
}

// Like std::convert::From, FromVersion should be reflexive.
// This allows it to be used in generic parameters where any
// version should be accepted.
impl<T> FromVersion<T> for T
where
    T: Versioned,
{
    fn from_version(t: T) -> Self {
        t
    }
}

/// Convert an older message version to a newer message version.
///
/// This is the inverse of [`FromVersion`]; see its documentation for more.
pub trait IntoVersion<T> {
    /// Convert from an older `Versioned` type to a newer `Versioned` type.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MyStructV1 {
        x: u64,
    }

    // This will usually be done with a proc macro,
    // e.g. #[derive(Versioned)]
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

    type MyStruct = MyStructV3;

    impl Versioned for MyStructV3 {
        const VER: u16 = 3;
        type Base = MyStruct;
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

    // FIXME: build a proc-macro that can derive this.
    impl FromVersion<MyStructV1> for MyStructV3 {
        fn from_version(msv1: MyStructV1) -> Self {
            let msv2 = MyStructV2::from_version(msv1);
            MyStructV3::from_version(msv2)
        }
    }

    #[test]
    fn test_adapter() {
        let x = MyStructV1 { x: 42 };
        let y = MyStructV2::from_version(x.clone());
        println!("{:?}", y);
        let z = MyStructV3::from_version(x);
        assert_eq!(z, MyStructV3 { x: 42, y: 0 })
    }
}
