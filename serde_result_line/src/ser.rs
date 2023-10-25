//! Serialization of structs into result lines using serde

use std::fmt::Display;

use serde::{
    ser::{SerializeMap, SerializeStruct, SerializeStructVariant},
    Serializer,
};

use crate::{NamedItem, ResultItem};

///
/// Takes a serializable struct and turns it into a result line.
/// It is important to say that serializing a struct into a result line only works on completely flat structs!
/// That means that nested structs are not supported, unless #[serde(flatten)] is used.
/// This method works on [HashMap]s and [BTreeMap]s as well however.
///
/// # Arguments
///
/// * `t`: The struct to serialize
///
/// Returns: The struct serialized into a result line.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// // Any of these variants work. Empty variants are ignored in output
/// // That means that [Option::None] is ignored as well.
/// #[derive(serde::Serialize)]
/// enum E {
///     A,
///     B(&'static str),
///     C(i64),
/// }
///
/// #[derive(serde::Serialize)]
/// struct Test {
///     a: &'static str,
///     b: i64,
///     #[serde(flatten)]
///     c: BTreeMap<&'static str, u16>,
///     d : bool,
///     e: E,
/// }
///
/// let mut t = Test {
///     a: "hello world",
///     b: -123423904,
///     c: BTreeMap::new(),
///     d: true,
///     e: E::C(12),
/// };
///
/// t.c.insert("map key", 100);
///
/// assert_eq!(serde_result_line::to_string(&t), Ok(r#"RESULT a="hello world" b=-123423904 "map key"=100 d=true e=12"#.to_string()));
/// ```
pub fn to_string<T: serde::Serialize>(t: &T) -> Result<String, Erra> {
    let mut ser = ResultLineStructurizer {
        current_name: None,
        output: vec![],
    };
    t.serialize(&mut ser)?;

    let mut s = "RESULT".to_owned();
    for item in ser.output {
        s.push(' ');
        s.push_str(&item.to_string())
    }

    Ok(s)
}

struct ResultLineStructurizer {
    current_name: Option<ResultItem>,
    output: Vec<NamedItem>,
}

impl ResultLineStructurizer {
    pub fn eat<T: Into<ResultItem>>(&mut self, t: T) -> ResultItem {
        if let Some(name) = self.current_name.take() {
            ResultItem::Named(Box::new(NamedItem::new(name, t.into())))
        } else {
            t.into()
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Erra {
    #[error("{0}")]
    Generic(String),
    #[error("unsupported input type \"{0}\"")]
    Unsupported(&'static str),
    #[error("unnamed item found")]
    UnnamedItem,
}

impl serde::ser::Error for Erra {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Generic(msg.to_string())
    }
}

impl<'a> Serializer for &'a mut ResultLineStructurizer {
    type Ok = ResultItem;

    type Error = Erra;

    type SerializeSeq = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTuple = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleStruct = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeTupleVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = serde::ser::Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v as isize))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v as isize))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v as isize))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v as isize))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v as usize))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v as usize))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v as usize))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v as usize))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(v))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(String::from_utf8_lossy(v).as_ref())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.eat(()))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Self::Error::Unsupported("seq"))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Self::Error::Unsupported("tuple"))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Self::Error::Unsupported("tuple struct"))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Self::Error::Unsupported("tuple variant"))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Self::Error::Unsupported("struct variant"))
    }
}

impl<'a> SerializeMap for &'a mut ResultLineStructurizer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let res_item = key.serialize(&mut **self)?;
        self.current_name = Some(res_item);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let result = value.serialize(&mut **self)?;
        match result {
            ResultItem::Named(mut item) if !item.value.is_empty() => {
                self.output.push(std::mem::take(&mut *item))
            }
            ResultItem::Named(_) => {}
            _ => return Err(Self::Error::UnnamedItem),
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ResultItem::Empty)
    }
}

impl<'a> SerializeStruct for &'a mut ResultLineStructurizer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        <Self as SerializeMap>::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(ResultItem::Empty)
    }
}

impl<'a> SerializeStructVariant for &'a mut ResultLineStructurizer {
    type Ok = <Self as Serializer>::Ok;

    type Error = <Self as Serializer>::Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        <Self as SerializeStruct>::serialize_field(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        <Self as SerializeStruct>::end(self)
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    #[test]
    fn serialization_test() {
        #[derive(serde::Serialize)]
        enum E {
            A,
            B(&'static str),
            C(i64),
        }

        #[derive(serde::Serialize)]
        struct Test {
            a: &'static str,
            b: i64,
            #[serde(flatten)]
            c: BTreeMap<&'static str, u16>,
            d: bool,
            e: String,
            f: E,
            g: E,
            h: E,
        }

        let mut t = Test {
            a: "hello world",
            b: -123423904,
            c: BTreeMap::new(),
            d: true,
            e: "this is an owned string with unicode".to_owned(),
            f: E::A,
            g: E::B("string in a variant"),
            h: E::C(12356),
        };

        t.c.insert("nowhitespace", 8123);
        t.c.insert("a key", 8123);
        t.c.insert("another key", 1850);
        t.c.insert("yet another key", 21850);

        println!("{}", super::to_string(&t).unwrap());

        assert_eq!(super::to_string(&t), Ok(r#"RESULT a="hello world" b=-123423904 "a key"=8123 "another key"=1850 nowhitespace=8123 "yet another key"=21850 d=true e="this is an owned string with unicode" g="string in a variant" h=12356"#.to_string()))
    }
}
