use crate::{WsonMap, WsonValue};
use crate::error::WsonSerializeError;
use std::fmt::Write;

pub fn serialize_wson(data: &WsonMap) -> Result<String, WsonSerializeError> {
    let mut result = String::with_capacity(1024);
    result.push_str("{\n");
    serialize_map(&mut result, data, "    ")?;
    result.push_str("\n}");
    Ok(result)
}

fn serialize_map(result: &mut String, data: &WsonMap, indent: &str) -> Result<(), WsonSerializeError> {
    for (i, (key, value)) in data.iter().enumerate() {
        if i > 0 {
            result.push_str(",\n\n");
        }
        write!(result, "{}{} = ", indent, key)?;
        serialize_value(result, value, indent)?;
    }
    Ok(())
}

fn serialize_list(result: &mut String, data: &[WsonValue], indent: &str) -> Result<(), WsonSerializeError> {
    for (i, item) in data.iter().enumerate() {
        if i > 0 {
            result.push_str(",\n");
        }
        result.push_str(indent);
        serialize_value(result, item, indent)?;
    }
    Ok(())
}

fn serialize_value(result: &mut String, value: &WsonValue, indent: &str) -> Result<(), WsonSerializeError> {
    match value {
        WsonValue::Null => result.push_str("null"),
        WsonValue::Bool(b) => write!(result, "{}", b)?,
        WsonValue::Int(i) => write!(result, "{}", i)?,
        WsonValue::Float(f) => write!(result, "{}", f)?,
        WsonValue::String(s) => write!(result, "\"{}\"", s)?,
        WsonValue::Date(s) | WsonValue::DateTime(s) => result.push_str(s),
        WsonValue::Version(v) => {
            for (i, part) in v.iter().enumerate() {
                if i > 0 {
                    result.push('.');
                }
                write!(result, "{}", part)?;
            }
        }
        WsonValue::Array(arr) => {
            result.push_str("[\n");
            serialize_list(result, arr, &format!("{}    ", indent))?;
            result.push_str("\n");
            result.push_str(indent);
            result.push(']');
        }
        WsonValue::Object(obj) => {
            result.push_str("{\n");
            serialize_map(result, obj, &format!("{}    ", indent))?;
            result.push_str("\n");
            result.push_str(indent);
            result.push('}');
        }
    }
    Ok(())
}
