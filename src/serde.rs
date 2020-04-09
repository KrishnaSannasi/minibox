use super::MiniBox;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<T: Serialize> Serialize for MiniBox<T> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        T::serialize(self, ser)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for MiniBox<T> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        T::deserialize(de).map(MiniBox::new)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serde_zst() {
        #[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
        pub struct Foo;

        let foo = Foo;
        let foo_bx = MiniBox::new(foo);

        assert_eq!(foo_bx, foo_bx);

        let ser = serde_json::to_string(&foo).unwrap();
        let miniser = serde_json::to_string(&foo_bx).unwrap();

        assert_eq!(ser, miniser);

        let foo_2: Foo = serde_json::from_str(&ser).unwrap();
        let foo_bx_2: MiniBox<Foo> = serde_json::from_str(&ser).unwrap();

        assert_eq!(foo, foo_2);
        assert_eq!(foo_bx, foo_bx_2);
        assert_eq!(*foo_bx, foo_2);
        assert_eq!(foo, *foo_bx_2);
    }

    #[test]
    fn serde_small() {
        #[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
        pub struct Foo {
            a: u8,
            b: u32,
        }

        let foo = Foo { a: 31, b: 421 };
        let foo_bx = MiniBox::new(foo);

        assert_eq!(foo_bx, foo_bx);

        let ser = serde_json::to_string(&foo).unwrap();
        let miniser = serde_json::to_string(&foo_bx).unwrap();

        assert_eq!(ser, miniser);

        let foo_2: Foo = serde_json::from_str(&ser).unwrap();
        let foo_bx_2: MiniBox<Foo> = serde_json::from_str(&ser).unwrap();

        assert_eq!(foo, foo_2);
        assert_eq!(foo_bx, foo_bx_2);
        assert_eq!(*foo_bx, foo_2);
        assert_eq!(foo, *foo_bx_2);
    }

    #[test]
    fn serde_large() {
        #[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
        pub struct Foo {
            a: u8,
            b: u32,
            c: usize,
        }

        let foo = Foo {
            a: 31,
            b: 421,
            c: 123,
        };
        let foo_bx = MiniBox::new(foo);

        assert_eq!(foo_bx, foo_bx);

        let ser = serde_json::to_string(&foo).unwrap();
        let miniser = serde_json::to_string(&foo_bx).unwrap();

        assert_eq!(ser, miniser);

        let foo_2: Foo = serde_json::from_str(&ser).unwrap();
        let foo_bx_2: MiniBox<Foo> = serde_json::from_str(&ser).unwrap();

        assert_eq!(foo, foo_2);
        assert_eq!(foo_bx, foo_bx_2);
        assert_eq!(*foo_bx, foo_2);
        assert_eq!(foo, *foo_bx_2);
    }
}
