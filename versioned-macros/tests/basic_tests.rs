use versioned::{FromVersion, IntoVersion, Versioned};

#[test]
fn basic() {
    pub type Basic = BasicV1;

    #[derive(Versioned)]
    pub struct BasicV1 {
        name: String,
        age: u32,
        friends: Vec<String>,
    }

    assert_eq!(BasicV1::VER, 1);

    #[derive(Debug, PartialEq, Versioned)]
    pub struct BasicV2 {
        name: String,
        age: u32,
        alive: bool,
        friends: Vec<String>,
    }

    assert_eq!(BasicV2::VER, 2);

    let gg = BasicV1 {
        name: "Galileo".to_string(),
        age: 456,
        friends: vec!["Cigoli".to_owned(), "Castelli".to_owned()],
    };

    impl FromVersion<BasicV1> for BasicV2 {
        fn from_version(v1: BasicV1) -> BasicV2 {
            BasicV2 {
                name: v1.name,
                age: v1.age,
                alive: true,
                friends: v1.friends,
            }
        }
    }

    let gg2: BasicV2 = gg.into_version();

    assert_eq!(
        gg2,
        BasicV2 {
            name: "Galileo".to_string(),
            age: 456,
            alive: true,
            friends: vec!["Cigoli".to_owned(), "Castelli".to_owned()],
        }
    )
}
