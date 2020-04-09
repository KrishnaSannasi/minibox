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
