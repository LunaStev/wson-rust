use crate::WsonValue;
use crate::error::WsonParseError;
use std::collections::BTreeMap;
use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;

pub type WsonMap = BTreeMap<String, WsonValue>;

pub fn parse_wson(input: &str) -> Result<WsonMap, WsonParseError> {
    let cleaned = remove_comments(input);
    convert_wson_to_map(&cleaned, 1, 1)
}

pub fn remove_comments(input: &str) -> String {
    let mut result = String::new();
    let mut in_block = false;

    for line in input.lines() {
        let mut l = line.to_string();
        if in_block {
            if let Some(end) = l.find("*/") {
                l = l[end + 2..].to_string();
                in_block = false;
            } else {
                continue;
            }
        }

        while let Some(start) = l.find("/*") {
            if let Some(end) = l[start + 2..].find("*/") {
                let before = &l[..start];
                let after = &l[start + 2 + end + 2..];
                l = format!("{}{}", before, after);
            } else {
                l.truncate(start);
                in_block = true;
                break;
            }
        }

        if let Some(index) = l.find("//") {
            l = &l[..index];
        } else if let Some(index) = l.find('#') {
            l = &l[..index];
        }

        if !l.trim().is_empty() {
            result.push_str(l.trim_end());
            result.push('\n');
        }
    }

    result
}

fn parse_value(value: &str, line: usize, column: usize) -> Result<WsonValue, WsonParseError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(WsonValue::Null);
    }

    if value.starts_with('"') && value.ends_with('"') {
        return Ok(WsonValue::String(value[1..value.len() - 1].to_string()));
    }

    if value.eq_ignore_ascii_case("true") {
        return Ok(WsonValue::Bool(true));
    }

    if value.eq_ignore_ascii_case("false") {
        return Ok(WsonValue::Bool(false));
    }

    if let Ok(i) = value.parse::<i64>() {
        return Ok(WsonValue::Int(i));
    }

    if let Ok(f) = value.parse::<f64>() {
        return Ok(WsonValue::Float(f));
    }

    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Ok(WsonValue::Date(date.to_string()));
    }

    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Ok(WsonValue::DateTime(dt.to_string()));
    }

    let version_re = Regex::new(r"^\d+(\.\d+)+$").unwrap();
    if version_re.is_match(value) {
        let version = value
            .split('.')
            .filter_map(|v| v.parse::<u32>().ok())
            .collect();
        return Ok(WsonValue::Version(version));
    }

    if value.starts_with('{') && value.ends_with('}') {
        return convert_wson_to_map(value, line, column).map(WsonValue::Object);
    }

    if value.starts_with('[') && value.ends_with(']') {
        return parse_array(value, line, column).map(WsonValue::Array);
    }

    Err(WsonParseError::new(format!("Invalid value: {}", value), Some(line), Some(column)))
}

fn parse_array(array_str: &str, line: usize, column: usize) -> Result<Vec<WsonValue>, WsonParseError> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut brace = 0;
    let mut bracket = 0;
    let inner = &array_str[1..array_str.len() - 1];

    for (i, ch) in inner.chars().enumerate() {
        match ch {
            '{' => brace += 1,
            '}' => brace -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            ',' if brace == 0 && bracket == 0 => {
                items.push(parse_value(&current, line, column + i)?);
                current.clear();
                continue;
            }
            _ => {}
        }
        current.push(ch);
    }

    if !current.trim().is_empty() {
        items.push(parse_value(&current, line, column + inner.len())?);
    }

    Ok(items)
}

pub fn convert_wson_to_map(wson_str: &str, start_line: usize, start_column: usize) -> Result<WsonMap, WsonParseError> {
    let content = wson_str.trim();
    if !content.starts_with('{') || !content.ends_with('}') {
        return Err(WsonParseError::new("WSON format must start and end with curly braces.", Some(start_line), Some(start_column)));
    }

    let inner = &content[1..content.len() - 1];
    let mut map = WsonMap::new();
    let mut key = String::new();
    let mut value = String::new();
    let mut in_key = true;
    let mut brace = 0;
    let mut bracket = 0;
    let mut line = start_line;
    let mut column = start_column;

    for (i, ch) in inner.chars().enumerate() {
        if ch == '\n' {
            line += 1;
            column = 0;
            continue;
        }

        match ch {
            '=' | ':' if in_key && brace == 0 && bracket == 0 => {
                in_key = false;
                continue;
            }
            '{' => brace += 1,
            '}' => brace -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            ',' if brace == 0 && bracket == 0 => {
                let parsed_value = parse_value(&value, line, column + i)?;
                map.insert(key.trim().to_string(), parsed_value);
                key.clear();
                value.clear();
                in_key = true;
                continue;
            }
            _ => {}
        }

        if in_key {
            key.push(ch);
        } else {
            value.push(ch);
        }
    }

    if !key.trim().is_empty() {
        let parsed_value = parse_value(&value, line, column + inner.len())?;
        map.insert(key.trim().to_string(), parsed_value);
    }

    Ok(map)
}
