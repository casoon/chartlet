//! Deserialization of the fields that take one of several JSON shapes: [`MainPath`] (a list of
//! ids or `{ "from", "to" }`), [`TimeValue`] (a number or a string) and [`ZoomBound`] (an index or
//! a timestamp).
//!
//! Each reads its whole value first and then picks the variant, like an untagged enum, with the
//! same message when none fits. The error is raised while the value itself is being read, so its
//! position is the end of that value; [`super::validate::json_pointer`] maps it back to the
//! value's path and moves the position in the message to where it has always been reported.
//!
//! [`MainPath`]: super::MainPath
//! [`TimeValue`]: crate::time::TimeValue
//! [`ZoomBound`]: super::ZoomBound

use std::fmt;
use std::marker::PhantomData;

use serde::Deserializer;
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde_json::Value;

/// The start of the message for a value that fits none of the shapes, followed by the type name.
pub(crate) const NO_VARIANT: &str = "data did not match any variant of untagged enum ";

/// One JSON value as read, before a variant is picked. Lists and objects are kept whole, every
/// member of an object in order and duplicate keys included.
pub(crate) enum Shape {
    /// `null`, `true` or `false`.
    Literal,
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    Text(String),
    List(Vec<Value>),
    Object(Vec<(String, Value)>),
}

/// Reads one value and hands it to `pick`; `None` from `pick` fails with [`NO_VARIANT`] and
/// `name`.
pub(crate) fn deserialize<'de, D: Deserializer<'de>, T>(
    deserializer: D,
    name: &'static str,
    pick: fn(Shape) -> Option<T>,
) -> Result<T, D::Error> {
    deserializer.deserialize_any(ShapeVisitor {
        name,
        pick,
        output: PhantomData,
    })
}

struct ShapeVisitor<T> {
    name: &'static str,
    pick: fn(Shape) -> Option<T>,
    output: PhantomData<T>,
}

impl<T> ShapeVisitor<T> {
    fn pick<E: Error>(&self, shape: Shape) -> Result<T, E> {
        (self.pick)(shape).ok_or_else(|| E::custom(format_args!("{NO_VARIANT}{}", self.name)))
    }
}

impl<'de, T> Visitor<'de> for ShapeVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.name)
    }

    fn visit_unit<E: Error>(self) -> Result<T, E> {
        self.pick(Shape::Literal)
    }

    fn visit_bool<E: Error>(self, _: bool) -> Result<T, E> {
        self.pick(Shape::Literal)
    }

    fn visit_u64<E: Error>(self, value: u64) -> Result<T, E> {
        self.pick(Shape::Unsigned(value))
    }

    fn visit_i64<E: Error>(self, value: i64) -> Result<T, E> {
        self.pick(Shape::Signed(value))
    }

    fn visit_f64<E: Error>(self, value: f64) -> Result<T, E> {
        self.pick(Shape::Float(value))
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<T, E> {
        self.pick(Shape::Text(value.to_owned()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<T, A::Error> {
        let mut items = Vec::new();
        while let Some(item) = seq.next_element()? {
            items.push(item);
        }
        self.pick(Shape::List(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<T, A::Error> {
        let mut members = Vec::new();
        while let Some(member) = map.next_entry()? {
            members.push(member);
        }
        self.pick(Shape::Object(members))
    }
}
