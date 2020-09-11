//! # Serde WASM Document
//!
//! A Serde deserializer for `web_sys::Document`s.
//!
//! Really, this crate is just a placeholder until I can do something much
//! lighter on allocations.
//!
//! ## Reasons to use this:
//!
//! * `DomParser` fully supports XML parsing and validation
//! * using builtin functions results in small code size (theoretically)
//!
//! ## Reasons not to use this:
//!
//! * requires preallocating the entire `Document` using `DomParser`
//! * `Document` traversal results in a LOT of allocations
//! * cannot shrink memory after deserialization (WASM limitation)
//!
//! ## Differences between other XML libraries
//!
//! Other Serde-compliant XML deserializers like `quick-xml` usually take an
//! attribute-first approach to deserialization, which is to say that a
//! `struct`'s fields represent attribute names. Data content is accessed
//! through a special `"$value"` field name. This usage is fine for
//! attribute-heavy documents, but for data-heavy structures, this usage results
//! in a lot of single field nested `struct`s. Take the following XML:
//!
//! ```xml
//! <root>
//! <data1>some data</data1>
//! <data2>other data</data2>
//! </root>
//! ```
//!
//! To represent this in `quick-xml`, you would have to create something like:
//!
//! ```
//! # extern crate serde;
//! # use serde::Deserialize;
//! #[derive(Deserialize)]
//! struct Root {
//!     data1: Data1,
//!     data2: Data2,
//! }
//!
//! #[derive(Deserialize)]
//! struct Data1 {
//!     #[serde(rename = "$value")]
//!     data: String
//! }
//!
//! #[derive(Deserialize)]
//! struct Data2 {
//!     #[serde(rename = "$value")]
//!     data: String
//! }
//! ```
//!
//! This crate uses a data-first representation (well, actually it currently
//! doesn't support parsing attributes at all). So the above XML could instead
//! be structured like this:
//!
//! ```
//! # extern crate serde;
//! # use serde::Deserialize;
//! #[derive(Deserialize)]
//! struct Root {
//!     data1: String,
//!     data2: String,
//! }
//! ```
mod de;
mod error;

pub use de::iter::{into_iter, ElementIter};
pub use de::{from_doc, from_elem, Deserializer};
pub use error::{Error, Result};
