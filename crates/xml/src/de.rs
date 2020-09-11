// lets have a look at serde!

use serde::de;
//use serde::de::Deserialize;
use serde::de::Visitor;
use serde::forward_to_deserialize_any;


use crate::xml;

impl de::Error for xml::Error {
    fn custom<T: std::fmt::Display>(_msg: T) -> Self {
        xml::Error::SomeErr
    }
}

struct Decoder<'a> {
    _input: &'a str,
}

impl<'a> Decoder<'a> {}

/// Implement a deserializer on top of my decoder
impl<'de, 'a> de::Deserializer<'de> for &'a mut Decoder<'de> {
    type Error = xml::Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!();
    }

    // lets just be lazy for now, but this is how we would do this
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
