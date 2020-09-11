//! Deserializer for WASM `Document`s and `Element`s.
//! Currently doesn't support attributes. Probably the easiest way
//! is to parse them before children, would just require more global
//! state.
#[macro_use]
mod macros;

#[cfg(test)]
mod tests;

pub mod iter;

use crate::error::{Error, Result};
use core::str::FromStr;
use serde::de::{self, Deserialize, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use web_sys::{Document, Element};

/// A `serde::de::Deserializer` for `web_sys::Element`.
pub struct Deserializer {
    /// Stack of elements currently parsing
    stack: Vec<Option<Element>>,
    /// Stack of names (for sequence parsing)
    names: Vec<String>,
}

impl Deserializer {
    fn from_elem(elem: Option<Element>) -> Self {
        Deserializer {
            stack: vec![elem],
            names: vec![],
        }
    }
}

/// Create a `Deserializer` from an `Element`
pub fn from_elem<'a, T>(elem: Element) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_elem(Some(elem));
    T::deserialize(&mut deserializer)
}

/// Create a `Deserializer` from a `Document`
pub fn from_doc<'a, T>(doc: &Document) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_elem(doc.document_element());
    T::deserialize(&mut deserializer)
}

/// helper parsing methods
impl<'de> Deserializer {
    /// Get the top element on the stack.
    /// The top element can be None if the element had no siblings
    /// when the parser moved to the next sibling, signifying that
    /// sequence or struct parsing should stop.
    fn element(&self) -> Result<&Element> {
        if let Some(Some(elem)) = self.stack.last() {
            Ok(elem)
        } else {
            Err(Error::NoElement)
        }
    }

    /// Get the text_content from the current element.
    /// Calling this function will move the parser forward.
    fn data(&mut self) -> Result<String> {
        let ret = self
            .element()?
            .text_content()
            .ok_or_else(|| Error::NoTextContent);
        // move the parser forward
        self.next_sibling();
        ret
    }

    /// Parse the data as some type.
    fn parse_type<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        // TODO more helpful error messages
        self.data()?.parse::<T>().map_err(|_| Error::ParseError)
    }

    /// Get the full name of the current element.
    /// Does not do any namespace things other than
    /// rebuilding the name.
    fn element_name(&self) -> Result<String> {
        let elem = self.element()?;
        if let Some(prefix) = elem.prefix() {
            Ok(prefix + ":" + &elem.local_name())
        } else {
            Ok(elem.local_name())
        }
    }

    /// Helper to check if we have a valid element on the top of the stack.
    fn has_element(&self) -> bool {
        self.element().is_ok()
    }

    /// Replace the top element on the stack with its next sibling.
    /// Does nothing if the top element is None.
    fn next_sibling(&mut self) {
        if let Some(Some(elem)) = self.stack.last() {
            let sibling = elem.next_element_sibling();
            self.stack.pop();
            self.stack.push(sibling);
        }
    }
}

/// `Deserializer` implementation
impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer {
    type Error = Error;

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let name = self.element_name()?;
        visitor.visit_str(&name)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.next_sibling();
        visitor.visit_none()
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // basically if this function is ever called, we are some
        // otherwise the visitor should default to none
        visitor.visit_some(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let name = self.element_name()?;
        self.names.push(name);
        let res = visitor.visit_seq(&mut self);
        self.names.pop();
        res
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.data()?)
    }

    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let child = self.element()?.first_element_child();
        self.stack.push(child);
        let res = visitor.visit_map(&mut self);
        self.stack.pop();
        // make sure to progress the parser or else
        // we will loop on this element forever
        self.next_sibling();
        res
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        if self.data()?.is_empty() {
            visitor.visit_unit()
        } else {
            Err(Error::NonEmptyUnit)
        }
    }

    // simple parsed data
    deserialize_type!(bool);
    deserialize_type!(char);
    deserialize_type!(f32);
    deserialize_type!(f64);
    deserialize_type!(i128);
    deserialize_type!(i16);
    deserialize_type!(i32);
    deserialize_type!(i64);
    deserialize_type!(i8);
    deserialize_type!(u128);
    deserialize_type!(u16);
    deserialize_type!(u32);
    deserialize_type!(u64);
    deserialize_type!(u8);

    // not yet implemented
    deserialize_unimplemented!(any);
    deserialize_unimplemented!(byte_buf);
    deserialize_unimplemented!(bytes);
    deserialize_unimplemented!(enum);
    deserialize_unimplemented!(map);
    deserialize_unimplemented!(newtype_struct);
    deserialize_unimplemented!(str);
    deserialize_unimplemented!(tuple);
    deserialize_unimplemented!(tuple_struct);
    deserialize_unimplemented!(unit_struct);
}

/// Accessor for mapped data.
/// Expects that the top of the element stack contains the child
/// element to iterate over, although that element can be `None`
/// if there were no children.
impl<'de, 'a> MapAccess<'de> for Deserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.has_element() {
            // this should end up calling `deserialize_identifier`
            // which doesn't move the parser forward
            seed.deserialize(&mut *self).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // this should end up calling some deserialize method
        // which does move the parser forward
        seed.deserialize(&mut *self)
    }
}

/// Accessor for sequenced data.
/// This should always yield at least one element since we only
/// call this accessor after seeing the first element in a list.
impl<'de, 'a> SeqAccess<'de> for Deserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if let Some(name) = self.element_name().ok() {
            // only parse if the element is part of our list
            if &name == self.names.last().unwrap() {
                return seed.deserialize(&mut *self).map(Some);
            }
        }
        // there was either no element or it wasn't part of our list
        Ok(None)
    }
}
