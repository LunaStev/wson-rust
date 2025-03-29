use crate::{WsonMap, WsonValue};
use crate::error::WsonSerializeError;

pub fn serialize_wson(data: &WsonMap) -> Result<String, WsonSerializeError> {
    let mut result = String::from("{\n");
    result.push_str(&serialize_map(data, "    ")?);
    result.push_str("\n}");
    Ok(result)
}

fn serialize_map(data: &WsonMap, indent: &str) -> Result<String, WsonSerializeError> {
    let mut result = String::new();

    for (i, (key, value)) in data.iter().enumerate() {
        if i > 0 {
            result.push_str(",\n\n");
        }
        result.push_str(&format!("{}{} = {}", indent, key, serialize_value(value, indent)?));
    }

    Ok(result)
}

fn serialize_list(data: &[WsonValue], indent: &str) -> Result<String, WsonSerializeError> {
    let mut result = String::new();

    for (i, item) in data.iter().enumerate() {
        if i > 0 {
            result.push_str(",\n");
        }
        result.push_str(indent);
        result.push_str(&serialize_value(item, indent)?);
    }

    Ok(result)
}

fn serialize_value(value: &WsonValue, indent: &str) -> Result<String, WsonSerializeError> {
    match value {
        WsonValue::Null => Ok("null".to_string()),
        WsonValue::Bool(b) => Ok(b.to_string()),
        WsonValue::Int(i) => Ok(i.to_string()),
        WsonValue::Float(f) => Ok(f.to_string()),
        WsonValue::String(s) => Ok(format!("\"{}\"", s)),
        WsonValue::Date(s) => Ok(s.clone()),
        WsonValue::DateTime(s) => Ok(s.clone()),
        WsonValue::Version(v) => Ok(v.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(".")),
        WsonValue::Array(arr) => {
            let nested = serialize_list(arr, &(indent.to_string() + "    "))?;
            Ok(format!("[\n{}\n{}]", nested, indent))
        }
        WsonValue::Object(obj) => {
            let inner = serialize_map(obj, &(indent.to_string() + "    "))?;
            Ok(format!("{{\n{}\n{}}}", inner, indent))
        }
    }
}
