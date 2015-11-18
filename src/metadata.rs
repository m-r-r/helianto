use std::any::Any;
use std::iter::{FromIterator, Iterator};
use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};
use super::{Error, Result};
use super::utils::DateTime;

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    DateTime(DateTime),
    Vec(Vec<Value>),
    Map(BTreeMap<String, Value>),
}

impl ToJson for Value {
    fn to_json(&self) -> Json {
        use self::Value::*;
        match *self {
            Null => Json::Null,
            Bool(value) => Json::Boolean(value),
            I64(value) => Json::I64(value),
            U64(value) => Json::U64(value),
            F64(value) => Json::F64(value),
            String(ref value) => Json::String(value.clone()),
            DateTime(ref value) => value.to_json(),
            Vec(ref vec) => Json::Array(vec.iter().map(|v| v.to_json()).collect()),
            Map(ref map) =>
                Json::Object(map.iter().map(|(k, v)| (k.clone(), v.to_json())).collect()),
        }
    }
}

impl<'l> From<&'l str> for Value {
    fn from(string: &str) -> Value {
        Value::String(string.into())
    }
}

impl<V> FromIterator<V> for Value where Value: From<V> {
    fn from_iter<I>(iterator: I) -> Self
        where I: IntoIterator<Item = V>
    {
        Value::Vec(iterator.into_iter().map(|v| Value::from(v)).collect())
    }
}

impl<K, V> FromIterator<(K, V)> for Value where Value: From<V>, K: Into<String> {
    fn from_iter<I>(iterator: I) -> Self
        where I: IntoIterator<Item = (K, V)>
    {
        Value::Map(iterator.into_iter().map(|(k, v)| (k.into(), Value::from(v))).collect())
    }
}

macro_rules! impl_from (
    ($from: ty, $variant: path) => (
        impl From<$from> for Value {
            fn from(value: $from) -> Value {
                $variant(value)
            }
        }
    )
);

pub trait OptAsRef<T> {
    fn as_ref<'l>(&'l self) -> Option<&'l T>;
    fn as_mut<'l>(&'l mut self) -> Option<&'l mut T>;
}

macro_rules! impl_opt_as_ref(
    ($to: ty, $variant: path) => (
        impl OptAsRef<$to> for Value {
            fn as_ref<'l>(&'l self) -> Option<&'l $to> {
                match *self {
                    $variant(ref value) => Some(value),
                    _ => None,
                }
            }

            fn as_mut<'l>(&'l mut self) -> Option<&'l mut $to> {
                match *self {
                    $variant(ref mut value) => Some(value),
                    _ => None,
                }
            }
        }
    ) 
);

macro_rules! impl_into (
    ($to: ty, $variant: path) => (
        impl Into<Option<$to>> for Value {
            fn into(self) -> Option<$to> {
                match self {
                    $variant(e) => Some(e),
                    _ => None,
                }
            }
        }
    )
);

impl_from!(bool, Value::Bool);
impl_from!(i64, Value::I64);
impl_from!(u64, Value::U64);
impl_from!(f64, Value::F64);
impl_from!(String, Value::String);
impl_from!(DateTime, Value::DateTime);
impl_from!(Vec<Value>, Value::Vec);
impl_from!(BTreeMap<String, Value>, Value::Map);

impl_opt_as_ref!(bool, Value::Bool);
impl_opt_as_ref!(i64, Value::I64);
impl_opt_as_ref!(u64, Value::U64);
impl_opt_as_ref!(f64, Value::F64);
impl_opt_as_ref!(String, Value::String);
impl_opt_as_ref!(DateTime, Value::DateTime);
impl_opt_as_ref!(Vec<Value>, Value::Vec);
impl_opt_as_ref!(BTreeMap<String, Value>, Value::Map);

impl_into!(bool, Value::Bool);
impl_into!(i64, Value::I64);
impl_into!(u64, Value::U64);
impl_into!(f64, Value::F64);
impl_into!(String, Value::String);
impl_into!(DateTime, Value::DateTime);
impl_into!(Vec<Value>, Value::Vec);
impl_into!(BTreeMap<String, Value>, Value::Map);


pub trait Field {
    fn get_name(&self) -> &'static str;
    fn from_raw(&self, raw: &str) -> Result<Value>;

    #[inline]
    fn get_default(&self) -> Option<Value> {
        None
    }
}


fn read_metadata_list(metadata: &str) -> Result<Value> {
    let sep = if let Some(_) = metadata.find(';') {
        ';'
    } else {
        ','
    };
    Ok(metadata.split(sep).map(|s| String::from(s.trim())).filter(|s| s.len() > 0).collect())
}


pub struct Text(pub &'static str);
impl Field for Text {

    fn get_name(&self) -> &'static str {
        self.0
    }

    fn from_raw(&self, raw: &str) -> Result<Value> {
        Ok(Value::from(raw.trim()))
    }
}

#[test]
fn test_text_from_raw() {
    let result = Text("title").from_raw("foo").ok();
}


pub struct Date(pub &'static str);
impl Field for Date {

    fn get_name(&self) -> &'static str {
        self.0
    }

    fn from_raw(&self, raw: &str) -> Result<Value> {
        DateTime::from_string(raw)
            .ok_or_else(|| Error::InvalidDate { date: raw.into() })
            .map(Value::from)
    }
}


pub struct Keywords(pub &'static str);
impl Field for Keywords {

    fn get_name(&self) -> &'static str {
        self.0
    }

    fn from_raw(&self, raw: &str) -> Result<Value> {
        read_metadata_list(raw)
    }
}
