use crate::WsonValue;
use crate::error::WsonParseError;
use std::collections::BTreeMap;
use chrono::{NaiveDate, NaiveDateTime};
use regex::Regex;
use once_cell::sync::Lazy;

static VERSION_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+(\.\d+)+$").unwrap());

pub type WsonMap = BTreeMap<String, WsonValue>;

pub fn parse_wson(input: &str) -> Result<WsonMap, WsonParseError> {
    let cleaned = remove_comments(input);
    convert_wson_to_map(&cleaned, 1, 1)
}

pub fn remove_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_block = false;

    for raw_line in input.lines() {
        let mut line = raw_line.to_string();

        if in_block {
            if let Some(end) = line.find("*/") {
                line = line[end + 2..].to_string();
                in_block = false;
            } else {
                continue;
            }
        }

        while let Some(start) = line.find("/*") {
            if let Some(end) = line[start + 2..].find("*/") {
                let end_pos = start + 2 + end + 2;
                let before = &line[..start];
                let after = &line[end_pos..];
                line = format!("{}{}", before, after);
            } else {
                line = line[..start].to_string();
                in_block = true;
                break;
            }
        }

        if let Some(index) = line.find("//") {
            line = line[..index].to_string();
        } else if let Some(index) = line.find('#') {
            line = line[..index].to_string();
        }

        let trimmed = line.trim_end();
        if !trimmed.is_empty() {
            result.push_str(trimmed);
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

    if value.eq_ignore_ascii_case("null") {
        return Ok(WsonValue::Null);
    }

    if let Ok(i) = value.parse::<i64>() {
        return Ok(WsonValue::Int(i));
    }

    if VERSION_RE.is_match(value) {
        let version = value
            .split('.')
            .filter_map(|v| v.parse::<u32>().ok())
            .collect();
        return Ok(WsonValue::Version(version));
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
