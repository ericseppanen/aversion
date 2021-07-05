# aversion

### Versioned data structures in Rust

**This crate is still under development.**

The goal of this crate is to make versioned data structures easy. For example,
imagine we start out with this struct:
```rust
struct FooV1 {
    val: u32,
}
```
If we serialize data to files in this format, but later discover that we want
to make a change:
```rust
struct FooV2 {
    val: u64,
}
```
... then we have a bunch of work to do, if we want to support our previous
files. We may need to increment a version number in the file header, and
possibly go through all the different places where `FooV1` was used, and decide
whether to upgrade it to `FooV2`. Any place a `FooV1` was used, we need to keep
that code around, or risk breaking compatibilty.

This crate adds traits that allow us to track the version of each struct and
derive traits and methods to allow upgrading any struct dynamically to the
latest version. This means that most code only ever needs to interact with the
latest version, while still retaining the ability to read older files.

To make this work, structs must follow a particular pattern:
- Versioned structs must follow the naming
  convention `Name` + `V` + `{integer}`, i.e. `FooV1` or `BarV42`.
- Versions must start at 1, and be contiguous.
- There must be a type alias `type Foo = FooV3` that points to the latest
  version.
- For each pair of versions, `N` and `N+1`, the trait `FromVersion` must be
  implemented. For example:
```rust

impl FromVersion<FooV1> for FooV2 {
    fn from_version(v1: FooV1) -> Self {
        FooV2 { val: v1.val.into() }
    }
}
```

This crate is still new, and these rules may evolve in the future.

#### Deserialization

We want to be able to deserialize data structures without knowing ahead of
time what version is stored.

To do this, we use the `DataSource` trait, to specify our serialization
format, header format, and error types.

Once the `UpgradeLatest` trait is implemented (there is a derive macro for
this), we can quickly deserialize any version of our data structure, e.g.
```
// Define a data source
let src = CborData::new(...);
// Read a the next header + message, and upgrade to the latest version
let msg: Foo = data_src.expect_message()?;
```
Note that `msg` in this example is always the latest version of the `Foo`
struct, regardless of which version was actually stored. As long as the
`FromVersion` code is correct, the rest of the program never needs to be aware
of which version was read from the file.

#### Message Groups

We can extend this logic to groups of different messages, to automatically
build a dispatch function. For example, if we define a collection of messages:
```rust
enum MyProtocol {
    Foo(Foo),
    Bar(Bar),
}
```

We can derive the trait `GroupDeserialize` that can automatically deserialize
any of the messages in `MyProtocol`:
```rust
let incoming_message = MyProtocol::read_message(&mut my_data_source)?;
match incoming_message {
    Foo(f) => {
        // handle the received Foo message
    }
    Bar(b) => {
        // handle the received Bar message
    }
}
```
Similar to the previous example, the header will tell us which message
was sent (i.e. `Foo` or `Bar`), along with the version of that struct (`FooV1`
or `FooV2`) and `read_message` deserializes the correct version of the struct,
upgrades it to the latest version, and returns it as a `MyProtocol`
enum, for the caller to handle.

License: Apache-2.0
