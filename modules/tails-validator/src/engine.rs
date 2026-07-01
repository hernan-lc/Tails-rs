use crate::formats;
use crate::helpers::{issue, path_issue, range_err, truncate, type_err, type_name};
use crate::types::{PathSegment, SchemaDef, ValidationIssue};

pub fn validate_value(
    schema: &SchemaDef,
    value: &serde_json::Value,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    match schema {
        SchemaDef::String {
            min_length,
            max_length,
            pattern,
            format,
            custom_error,
        } => validate_string(
            value,
            *min_length,
            *max_length,
            pattern.as_deref(),
            format.as_deref(),
            custom_error.as_deref(),
        ),
        SchemaDef::Number {
            min,
            max,
            integer,
            positive,
            negative,
            multiple_of,
            finite,
            custom_error,
        } => validate_number(
            value,
            *min,
            *max,
            *integer,
            *positive,
            *negative,
            *multiple_of,
            *finite,
            custom_error.as_deref(),
        ),
        SchemaDef::Boolean { custom_error } => validate_boolean(value, custom_error.as_deref()),
        SchemaDef::Null { custom_error } => validate_null(value, custom_error.as_deref()),
        SchemaDef::Undefined => validate_undefined(value),
        SchemaDef::Any | SchemaDef::Unknown => Ok(value.clone()),
        SchemaDef::Literal {
            value: expected,
            custom_error,
        } => validate_literal(value, expected, custom_error.as_deref()),
        SchemaDef::Enum {
            values,
            custom_error,
        } => validate_enum(value, values, custom_error.as_deref()),
        SchemaDef::Object {
            properties,
            required,
            strict,
            custom_error,
        } => validate_object(
            value,
            properties,
            required,
            *strict,
            custom_error.as_deref(),
        ),
        SchemaDef::Array {
            items,
            min_length,
            max_length,
            unique_items,
            custom_error,
        } => validate_array(
            value,
            items,
            *min_length,
            *max_length,
            *unique_items,
            custom_error.as_deref(),
        ),
        SchemaDef::Tuple {
            schemas,
            custom_error,
        } => validate_tuple(value, schemas, custom_error.as_deref()),
        SchemaDef::Record {
            values,
            custom_error,
        } => validate_record(value, values, custom_error.as_deref()),
        SchemaDef::Union {
            schemas,
            custom_error,
        } => validate_union(value, schemas, custom_error.as_deref()),
        SchemaDef::Intersection {
            schemas,
            custom_error,
        } => validate_intersection(value, schemas, custom_error.as_deref()),
        SchemaDef::Optional { inner } => validate_optional(value, inner),
        SchemaDef::Nullable { inner } => validate_nullable(value, inner),
        SchemaDef::DefaultValue { inner, default } => validate_default(value, inner, default),
        SchemaDef::Coerce { target, inner } => validate_coerce(value, target, inner),
        SchemaDef::Refine { inner, message } => validate_refine(value, inner, message.as_deref()),
        SchemaDef::Transform { inner, transform } => validate_transform(value, inner, transform),
        SchemaDef::Lazy { schema, .. } => validate_value(schema, value),
        SchemaDef::Pipeline {
            schemas,
            custom_error,
        } => validate_pipeline(value, schemas, custom_error.as_deref()),
        SchemaDef::Preprocess { transform, inner } => validate_preprocess(value, transform, inner),
    }
}

fn validate_string(
    value: &serde_json::Value,
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<&str>,
    format: Option<&str>,
    custom_error: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let s = match value.as_str() {
        Some(s) => s,
        None => return Err(vec![type_err("string", value, custom_error)]),
    };

    let mut issues = Vec::new();

    if let Some(min) = min_length {
        if s.len() < min {
            issues.push(range_err(
                "too_small",
                custom_error.unwrap_or(&format!(
                    "String must contain at least {} character(s)",
                    min
                )),
                &format!(">= {} chars", min),
                &format!("{} chars", s.len()),
            ));
        }
    }
    if let Some(max) = max_length {
        if s.len() > max {
            issues.push(range_err(
                "too_big",
                custom_error
                    .unwrap_or(&format!("String must contain at most {} character(s)", max)),
                &format!("<= {} chars", max),
                &format!("{} chars", s.len()),
            ));
        }
    }
    if let Some(pat) = pattern {
        match fancy_regex::Regex::new(pat) {
            Ok(re) if !re.is_match(s).unwrap_or(false) => {
                issues.push(range_err(
                    "invalid_string",
                    custom_error.unwrap_or(&format!("String does not match pattern: {}", pat)),
                    &format!("pattern: {}", pat),
                    &format!("\"{}\"", truncate(s, 50)),
                ));
            }
            Err(_) => issues.push(issue(
                "invalid_schema",
                &format!("Invalid regex pattern: {}", pat),
                "valid regex",
                pat,
            )),
            _ => {}
        }
    }
    if let Some(fmt) = format {
        let fmt_err = custom_error.unwrap_or(match fmt {
            "email" => "Invalid email address",
            "url" => "Invalid URL",
            "uuid" => "Invalid UUID",
            "date" | "datetime" => "Invalid date/datetime string",
            "ipv4" => "Invalid IPv4 address",
            "ipv6" => "Invalid IPv6 address",
            "phone" => "Invalid phone number",
            "base64" => "Invalid base64 string",
            _ => "Invalid format",
        });
        let valid = match fmt {
            "email" => formats::is_valid_email(s),
            "url" => formats::is_valid_url(s),
            "uuid" => formats::is_valid_uuid(s),
            "date" | "datetime" => formats::is_valid_datetime(s),
            "ipv4" => formats::is_valid_ipv4(s),
            "ipv6" => formats::is_valid_ipv6(s),
            "phone" => formats::is_valid_phone(s),
            "base64" => formats::is_valid_base64(s),
            _ => true,
        };
        if !valid {
            issues.push(range_err(
                "invalid_format",
                fmt_err,
                fmt,
                &format!("\"{}\"", truncate(s, 50)),
            ));
        }
    }

    if issues.is_empty() {
        Ok(value.clone())
    } else {
        Err(issues)
    }
}

fn validate_number(
    value: &serde_json::Value,
    min: Option<f64>,
    max: Option<f64>,
    integer: bool,
    positive: bool,
    negative: bool,
    multiple_of: Option<f64>,
    finite: bool,
    custom_error: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let n = match value.as_f64() {
        Some(n) => n,
        None => return Err(vec![type_err("number", value, custom_error)]),
    };

    let mut issues = Vec::new();
    if finite && !n.is_finite() {
        issues.push(range_err(
            "not_finite",
            custom_error.unwrap_or("Number must be finite"),
            "finite number",
            &n.to_string(),
        ));
    }
    if integer && (n.fract() != 0.0 || n.abs() > (i64::MAX as f64)) {
        issues.push(range_err(
            "not_integer",
            custom_error.unwrap_or("Expected integer"),
            "integer",
            &n.to_string(),
        ));
    }
    if let Some(v) = min {
        if n < v {
            issues.push(range_err(
                "too_small",
                custom_error.unwrap_or(&format!("Number must be >= {}", v)),
                &format!(">= {}", v),
                &n.to_string(),
            ));
        }
    }
    if let Some(v) = max {
        if n > v {
            issues.push(range_err(
                "too_big",
                custom_error.unwrap_or(&format!("Number must be <= {}", v)),
                &format!("<= {}", v),
                &n.to_string(),
            ));
        }
    }
    if positive && n <= 0.0 {
        issues.push(range_err(
            "not_positive",
            custom_error.unwrap_or("Number must be positive"),
            "> 0",
            &n.to_string(),
        ));
    }
    if negative && n >= 0.0 {
        issues.push(range_err(
            "not_negative",
            custom_error.unwrap_or("Number must be negative"),
            "< 0",
            &n.to_string(),
        ));
    }
    if let Some(d) = multiple_of {
        if d == 0.0 {
            issues.push(issue(
                "invalid_schema",
                "multipleOf cannot be 0",
                "non-zero number",
                "0",
            ));
        } else if (n / d).fract() != 0.0 {
            issues.push(range_err(
                "not_multiple_of",
                custom_error.unwrap_or(&format!("Number must be a multiple of {}", d)),
                &format!("multiple of {}", d),
                &n.to_string(),
            ));
        }
    }

    if issues.is_empty() {
        Ok(value.clone())
    } else {
        Err(issues)
    }
}

fn validate_boolean(
    value: &serde_json::Value,
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    if value.is_boolean() {
        Ok(value.clone())
    } else {
        Err(vec![type_err("boolean", value, ce)])
    }
}

fn validate_null(
    value: &serde_json::Value,
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    if value.is_null() {
        Ok(value.clone())
    } else {
        Err(vec![type_err("null", value, ce)])
    }
}

fn validate_undefined(
    value: &serde_json::Value,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    if value.is_null() {
        Ok(serde_json::Value::Null)
    } else {
        Err(vec![issue(
            "invalid_type",
            "Expected undefined",
            "undefined",
            &type_name(value),
        )])
    }
}

fn validate_literal(
    value: &serde_json::Value,
    expected: &serde_json::Value,
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    if value == expected {
        Ok(value.clone())
    } else {
        Err(vec![range_err(
            "invalid_literal",
            ce.unwrap_or(&format!("Expected literal {}", expected)),
            &expected.to_string(),
            &value.to_string(),
        )])
    }
}

fn validate_enum(
    value: &serde_json::Value,
    values: &[serde_json::Value],
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    if values.contains(value) {
        Ok(value.clone())
    } else {
        let exp = values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" | ");
        Err(vec![range_err(
            "invalid_enum",
            ce.unwrap_or(&format!("Expected one of: {}", exp)),
            &exp,
            &value.to_string(),
        )])
    }
}

fn validate_object(
    value: &serde_json::Value,
    properties: &std::collections::HashMap<String, SchemaDef>,
    required: &[String],
    strict: bool,
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let obj = match value.as_object() {
        Some(o) => o,
        None => return Err(vec![type_err("object", value, ce)]),
    };
    let mut issues = Vec::new();
    let mut output = serde_json::Map::new();

    for field in required {
        if !obj.contains_key(field) {
            issues.push(path_issue(
                "missing",
                &format!("Required field '{}' is missing", field),
                "present",
                "missing",
                field,
            ));
        }
    }
    for (key, schema) in properties {
        if let Some(val) = obj.get(key) {
            match validate_value(schema, val) {
                Ok(validated) => {
                    output.insert(key.clone(), validated);
                }
                Err(mut field_issues) => {
                    for issue in &mut field_issues {
                        let mut path = vec![PathSegment { key: key.clone() }];
                        if let Some(ref existing) = issue.path {
                            path.extend(existing.iter().cloned());
                        }
                        issue.path = Some(path);
                    }
                    issues.extend(field_issues);
                }
            }
        } else if !required.contains(key) {
            match schema {
                SchemaDef::Optional { .. } | SchemaDef::Nullable { .. } => {}
                SchemaDef::DefaultValue { default, .. } => {
                    output.insert(key.clone(), default.clone());
                }
                _ => {}
            }
        }
    }
    if strict {
        for key in obj.keys() {
            if !properties.contains_key(key) {
                issues.push(path_issue(
                    "unrecognized",
                    &format!("Unexpected property '{}'", key),
                    "unknown property",
                    &format!("property '{}'", key),
                    key,
                ));
            }
        }
    }
    if issues.is_empty() {
        Ok(serde_json::Value::Object(output))
    } else {
        Err(issues)
    }
}

fn validate_array(
    value: &serde_json::Value,
    items: &SchemaDef,
    min_length: Option<usize>,
    max_length: Option<usize>,
    unique_items: bool,
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let arr = match value.as_array() {
        Some(a) => a,
        None => return Err(vec![type_err("array", value, ce)]),
    };
    let mut issues = Vec::new();

    if let Some(min) = min_length {
        if arr.len() < min {
            issues.push(range_err(
                "too_small",
                ce.unwrap_or(&format!("Array must contain at least {} element(s)", min)),
                &format!(">= {} items", min),
                &format!("{} items", arr.len()),
            ));
        }
    }
    if let Some(max) = max_length {
        if arr.len() > max {
            issues.push(range_err(
                "too_big",
                ce.unwrap_or(&format!("Array must contain at most {} element(s)", max)),
                &format!("<= {} items", max),
                &format!("{} items", arr.len()),
            ));
        }
    }
    if unique_items {
        let mut seen = std::collections::HashSet::new();
        for (i, item) in arr.iter().enumerate() {
            let key = item.to_string();
            if !seen.insert(key) {
                issues.push(path_issue(
                    "unique_items",
                    &format!("Array contains duplicate item at index {}", i),
                    "unique items",
                    &format!("duplicate at index {}", i),
                    &i.to_string(),
                ));
            }
        }
    }

    let mut output = Vec::new();
    for (i, item) in arr.iter().enumerate() {
        match validate_value(items, item) {
            Ok(validated) => output.push(validated),
            Err(mut item_issues) => {
                for issue in &mut item_issues {
                    let mut path = vec![PathSegment { key: i.to_string() }];
                    if let Some(ref existing) = issue.path {
                        path.extend(existing.iter().cloned());
                    }
                    issue.path = Some(path);
                }
                issues.extend(item_issues);
            }
        }
    }
    if issues.is_empty() {
        Ok(serde_json::Value::Array(output))
    } else {
        Err(issues)
    }
}

fn validate_tuple(
    value: &serde_json::Value,
    schemas: &[SchemaDef],
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let arr = match value.as_array() {
        Some(a) => a,
        None => return Err(vec![type_err("tuple", value, ce)]),
    };
    if arr.len() != schemas.len() {
        return Err(vec![range_err(
            "invalid_tuple",
            ce.unwrap_or(&format!(
                "Expected tuple of length {}, got {}",
                schemas.len(),
                arr.len()
            )),
            &format!("tuple of length {}", schemas.len()),
            &format!("tuple of length {}", arr.len()),
        )]);
    }
    let mut issues = Vec::new();
    let mut output = Vec::new();
    for (i, (item, schema)) in arr.iter().zip(schemas.iter()).enumerate() {
        match validate_value(schema, item) {
            Ok(validated) => output.push(validated),
            Err(mut item_issues) => {
                for issue in &mut item_issues {
                    let mut path = vec![PathSegment { key: i.to_string() }];
                    if let Some(ref existing) = issue.path {
                        path.extend(existing.iter().cloned());
                    }
                    issue.path = Some(path);
                }
                issues.extend(item_issues);
            }
        }
    }
    if issues.is_empty() {
        Ok(serde_json::Value::Array(output))
    } else {
        Err(issues)
    }
}

fn validate_record(
    value: &serde_json::Value,
    values_schema: &SchemaDef,
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let obj = match value.as_object() {
        Some(o) => o,
        None => return Err(vec![type_err("record", value, ce)]),
    };
    let mut issues = Vec::new();
    let mut output = serde_json::Map::new();
    for (key, val) in obj {
        match validate_value(values_schema, val) {
            Ok(validated) => {
                output.insert(key.clone(), validated);
            }
            Err(mut val_issues) => {
                for issue in &mut val_issues {
                    let mut path = vec![PathSegment { key: key.clone() }];
                    if let Some(ref existing) = issue.path {
                        path.extend(existing.iter().cloned());
                    }
                    issue.path = Some(path);
                }
                issues.extend(val_issues);
            }
        }
    }
    if issues.is_empty() {
        Ok(serde_json::Value::Object(output))
    } else {
        Err(issues)
    }
}

fn validate_union(
    value: &serde_json::Value,
    schemas: &[SchemaDef],
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let mut all_errors = Vec::new();
    for schema in schemas {
        match validate_value(schema, value) {
            Ok(validated) => return Ok(validated),
            Err(errors) => all_errors.push(errors),
        }
    }
    let msgs: Vec<String> = all_errors
        .iter()
        .enumerate()
        .map(|(i, e)| {
            format!(
                "Option {}: {}",
                i + 1,
                e.first().map(|e| e.message.as_str()).unwrap_or("invalid")
            )
        })
        .collect();
    Err(vec![range_err(
        "invalid_union",
        ce.unwrap_or(&format!("No union variant matched: {}", msgs.join("; "))),
        &format!("one of {} schemas", schemas.len()),
        &type_name(value),
    )])
}

fn validate_intersection(
    value: &serde_json::Value,
    schemas: &[SchemaDef],
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let mut issues = Vec::new();
    let mut output = value.clone();
    for schema in schemas {
        match validate_value(schema, &output) {
            Ok(validated) => {
                if let (Some(base), Some(ext)) = (output.as_object_mut(), validated.as_object()) {
                    for (k, v) in ext {
                        base.insert(k.clone(), v.clone());
                    }
                } else {
                    output = validated;
                }
            }
            Err(errors) => issues.extend(errors),
        }
    }
    if issues.is_empty() {
        Ok(output)
    } else {
        if let Some(msg) = ce {
            return Err(vec![issue("custom", msg, "", "")]);
        }
        Err(issues)
    }
}

fn validate_optional(
    value: &serde_json::Value,
    inner: &SchemaDef,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    if value.is_null() {
        Ok(serde_json::Value::Null)
    } else {
        validate_value(inner, value)
    }
}

fn validate_nullable(
    value: &serde_json::Value,
    inner: &SchemaDef,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    if value.is_null() {
        Ok(serde_json::Value::Null)
    } else {
        validate_value(inner, value)
    }
}

fn validate_default(
    value: &serde_json::Value,
    inner: &SchemaDef,
    default: &serde_json::Value,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    if value.is_null() {
        validate_value(inner, default)
    } else {
        validate_value(inner, value)
    }
}

fn validate_coerce(
    value: &serde_json::Value,
    target: &str,
    inner: &SchemaDef,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let coerced = match target {
        "number" => {
            if let Some(s) = value.as_str() {
                s.parse::<f64>()
                    .map(|n| serde_json::json!(n))
                    .unwrap_or_else(|_| serde_json::Value::Null)
            } else if let Some(b) = value.as_bool() {
                serde_json::json!(if b { 1.0 } else { 0.0 })
            } else {
                return Err(vec![issue(
                    "invalid_coerce",
                    "Cannot coerce value to number",
                    "string or boolean",
                    &type_name(value),
                )]);
            }
        }
        "string" => serde_json::Value::String(match value {
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Null => "null".to_string(),
            other => other.to_string(),
        }),
        "boolean" => {
            if let Some(b) = value.as_bool() {
                serde_json::Value::Bool(b)
            } else if let Some(n) = value.as_f64() {
                serde_json::Value::Bool(n != 0.0)
            } else if let Some(s) = value.as_str() {
                serde_json::Value::Bool(!s.is_empty() && s != "0" && s != "false")
            } else {
                return Err(vec![issue(
                    "invalid_coerce",
                    "Cannot coerce value to boolean",
                    "coercible value",
                    &type_name(value),
                )]);
            }
        }
        _ => {
            return Err(vec![issue(
                "invalid_coerce",
                &format!("Unknown coercion target: {}", target),
                "number, string, or boolean",
                target,
            )])
        }
    };
    validate_value(inner, &coerced)
}

fn validate_refine(
    value: &serde_json::Value,
    inner: &SchemaDef,
    _message: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    validate_value(inner, value)
}

fn validate_transform(
    value: &serde_json::Value,
    inner: &SchemaDef,
    transform: &str,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let validated = validate_value(inner, value)?;
    match transform {
        "uppercase" => Ok(serde_json::Value::String(
            validated
                .as_str()
                .map(|s| s.to_uppercase())
                .unwrap_or_default(),
        )),
        "lowercase" => Ok(serde_json::Value::String(
            validated
                .as_str()
                .map(|s| s.to_lowercase())
                .unwrap_or_default(),
        )),
        "trim" => Ok(serde_json::Value::String(
            validated
                .as_str()
                .map(|s| s.trim().to_string())
                .unwrap_or_default(),
        )),
        "trim_start" => Ok(serde_json::Value::String(
            validated
                .as_str()
                .map(|s| s.trim_start().to_string())
                .unwrap_or_default(),
        )),
        "trim_end" => Ok(serde_json::Value::String(
            validated
                .as_str()
                .map(|s| s.trim_end().to_string())
                .unwrap_or_default(),
        )),
        "capitalize" => Ok(serde_json::Value::String(
            validated
                .as_str()
                .map(|s| {
                    let mut c = s.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .unwrap_or_default(),
        )),
        "to_number" => {
            if let Some(s) = validated.as_str() {
                s.parse::<f64>()
                    .map(|n| serde_json::json!(n))
                    .ok()
                    .map(Ok)
                    .unwrap_or_else(|| {
                        Err(vec![issue(
                            "transform_failed",
                            &format!("Cannot transform '{}' to number", truncate(s, 30)),
                            "numeric string",
                            &format!("\"{}\"", truncate(s, 30)),
                        )])
                    })
            } else {
                Ok(validated)
            }
        }
        "to_string" => Ok(serde_json::Value::String(validated.to_string())),
        "abs" => Ok(validated
            .as_f64()
            .map(|n| serde_json::json!(n.abs()))
            .unwrap_or(validated)),
        "round" => Ok(validated
            .as_f64()
            .map(|n| serde_json::json!(n.round()))
            .unwrap_or(validated)),
        "floor" => Ok(validated
            .as_f64()
            .map(|n| serde_json::json!(n.floor()))
            .unwrap_or(validated)),
        "ceil" => Ok(validated
            .as_f64()
            .map(|n| serde_json::json!(n.ceil()))
            .unwrap_or(validated)),
        _ => Ok(validated),
    }
}

fn validate_pipeline(
    value: &serde_json::Value,
    schemas: &[SchemaDef],
    ce: Option<&str>,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let mut current = value.clone();
    for schema in schemas {
        current = validate_value(schema, &current).map_err(|mut issues| {
            if let Some(msg) = ce {
                issues.clear();
                issues.push(issue("pipeline", msg, "", ""));
            }
            issues
        })?;
    }
    Ok(current)
}

fn validate_preprocess(
    value: &serde_json::Value,
    transform: &str,
    inner: &SchemaDef,
) -> Result<serde_json::Value, Vec<ValidationIssue>> {
    let preprocessed = match transform {
        "trim_strings" => {
            if let Some(obj) = value.as_object() {
                let mut result = serde_json::Map::new();
                for (k, v) in obj {
                    result.insert(
                        k.clone(),
                        v.as_str()
                            .map(|s| serde_json::Value::String(s.trim().to_string()))
                            .unwrap_or_else(|| v.clone()),
                    );
                }
                serde_json::Value::Object(result)
            } else {
                value.clone()
            }
        }
        "to_number" => value
            .as_str()
            .and_then(|s| s.parse::<f64>().ok())
            .map(|n| serde_json::json!(n))
            .unwrap_or_else(|| value.clone()),
        _ => value.clone(),
    };
    validate_value(inner, &preprocessed)
}
