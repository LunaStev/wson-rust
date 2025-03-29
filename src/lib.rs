pub mod error;
pub mod parser;
pub mod serializer;

use error::{WsonParseError, WsonSerializeError};
use parser::parse_wson;
use serializer::serialize_wson;
use std::collections::BTreeMap;

pub type WsonMap = BTreeMap<String, WsonValue>;

#[derive(Debug, Clone, PartialEq)]
pub enum WsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Date(String),
    DateTime(String),
    Version(Vec<u32>),
    Array(Vec<WsonValue>),
    Object(WsonMap),
}

pub fn loads(input: &str) -> Result<WsonMap, WsonParseError> {
    parse_wson(input)
}

pub fn dumps(data: &WsonMap) -> Result<String, WsonSerializeError> {
    serialize_wson(data)
}

pub fn validate(input: &str) -> bool {
    parse_wson(input).is_ok()
}
