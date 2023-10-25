//! A library for serializing and deserializing result lines.
//!
//! The result line is a format for conveniently inputting output data into Timo Bingmann's `sqlplot-tools`,
//! which can be found [here](https://github.com/bingmann/sqlplot-tools).
//! This is part of a Rust port of his tool, mainly developed for fun.
//!
//! This library provides [serde](https://docs.rs/serde/latest/serde/) serialization for result lines.
//! It also provides a (non-serde) deserializer which can create any type which implements [`FromIterator`]
//! for iterators over items of `(&str, ResultItem)`,
//! like `HashMap<&str, ResultItem>` or `Vec<(&str, ResultItem)>`.

use serde::Serialize;
use std::fmt::Display;

pub use de::from_string;
pub use ser::to_string;

mod de;
mod ser;

/// An enum representing the possible types a result item's value can be.
#[derive(Debug, Serialize, Clone, PartialEq, Default)]
pub enum ResultItem {
    /// A named item, e.g. `a="some value"`
    Named(Box<NamedItem>),
    /// An integer, e.g. `123`
    Integer(isize),
    /// A float, e.g. `123.456`
    Float(f64),
    /// A boolean, e.g. `true`
    Boolean(bool),
    /// A character, e.g. `'a'`
    Character(char),
    /// A text string, e.g. `"some value"`
    Text(String),
    /// An empty item
    #[default]
    Empty,
}

impl Display for ResultItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ResultItem as E;
        match self {
            E::Named(item) => write!(f, "{item}"),
            E::Integer(item) => write!(f, "{item}"),
            E::Float(item) => write!(f, "{item}"),
            E::Boolean(item) => write!(f, "{item}"),
            E::Character(item) => write!(f, "{item}"),
            E::Text(item) => write!(f, "{item}"),
            E::Empty => write!(f, ""),
        }
    }
}

/// A named item, e.g. `a="some value"`
#[derive(Debug, Serialize, Default, Clone, PartialEq)]
pub struct NamedItem {
    /// The name of the item
    name: ResultItem,
    /// The value of the item
    value: ResultItem,
}

impl NamedItem {
    pub fn new(name: impl Into<ResultItem>, value: impl Into<ResultItem>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl Display for NamedItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            ResultItem::Text(t) if t.contains(|c: char| c.is_whitespace()) => {
                write!(f, "\"{t}\"=")?
            }
            _ => write!(f, "{}=", &self.name)?,
        }
        match &self.value {
            ResultItem::Text(t) if t.contains(|c: char| c.is_whitespace()) => {
                write!(f, "\"{t}\"")
            }
            _ => write!(f, "{}", &self.value),
        }
    }
}

impl ResultItem {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }
}

impl From<usize> for ResultItem {
    fn from(value: usize) -> Self {
        Self::Integer(value as isize)
    }
}

impl From<isize> for ResultItem {
    fn from(value: isize) -> Self {
        Self::Integer(value)
    }
}

impl From<f64> for ResultItem {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for ResultItem {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<char> for ResultItem {
    fn from(value: char) -> Self {
        Self::Character(value)
    }
}

impl From<&'_ str> for ResultItem {
    fn from(value: &'_ str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<()> for ResultItem {
    fn from(_: ()) -> Self {
        Self::Empty
    }
}
