use {
    crate::syntax::{Id, IdError},
    std::panic::catch_unwind,
};

macro_rules! case {
    (b $s:literal) => {
        TryInto::<Id>::try_into($s as &[u8])
    };
    (v $s:literal) => {
        TryInto::<Id>::try_into($s.as_bytes().to_owned())
    };
    ($s:literal) => {
        TryInto::<Id>::try_into($s)
    };
}
#[test]
fn try_from_str() {
    assert_eq!(case!("hello").unwrap().as_str(), "hello");
    assert_eq!(case!("hello2").unwrap().as_str(), "hello2");
    assert_eq!(case!("h\"o2").err(), Some(IdError::InvalidIdChar(b'\"')));
    assert_eq!(case!("hello\0").err(), Some(IdError::InvalidIdChar(0)));
    assert_eq!(case!("hello😀").err(), Some(IdError::InvalidIdChar(240)));
}

#[test]
fn try_from_u8() {
    assert_eq!(case!(b b"hello").unwrap().as_str(), "hello");
    assert_eq!(case!(b b"hello2").unwrap().as_str(), "hello2");
    assert_eq!(case!(b b"h\"o2").err(), Some(IdError::InvalidIdChar(b'\"')));
    assert_eq!(case!(b b"hello\0").err(), Some(IdError::InvalidIdChar(0)));
}

#[test]
fn try_from_vec() {
    assert_eq!(case!(v "hello").unwrap().as_str(), "hello");
    assert_eq!(case!(v "hello2").unwrap().as_str(), "hello2");
    assert_eq!(case!(v "h\"o2").err(), Some(IdError::InvalidIdChar(b'\"')));
    assert_eq!(case!(v "hello\0").err(), Some(IdError::InvalidIdChar(0)));
    assert_eq!(case!(v "hello😀").err(), Some(IdError::InvalidIdChar(240)));
}

#[test]
fn literal() {
    assert_eq!(Id::literal("hello").as_str(), "hello");
    assert_eq!(Id::literal("hello2").as_str(), "hello2");
    assert!(catch_unwind(|| Id::literal("h\"o2")).is_err());
    assert!(catch_unwind(|| Id::literal("hello\0")).is_err());
    assert!(catch_unwind(|| Id::literal("hello😀")).is_err());
}
