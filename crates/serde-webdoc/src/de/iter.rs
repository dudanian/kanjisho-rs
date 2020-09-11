use super::{from_elem, Result};
use core::iter::Iterator;
use std::marker::PhantomData;
use web_sys::Element;

pub struct ElementIter<T> {
    elem: Option<Element>,
    phantom: PhantomData<T>,
}

impl<'de, T> Iterator for ElementIter<T>
where
    T: serde::Deserialize<'de>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(elem) = self.elem.take() {
            self.elem = elem.next_element_sibling();
            Some(from_elem::<T>(elem))
        } else {
            None
        }
    }
}

pub fn into_iter<'de, T>(elem: Option<Element>) -> ElementIter<T>
where
    T: serde::Deserialize<'de>,
{
    ElementIter::<T> {
        elem,
        phantom: PhantomData,
    }
}
