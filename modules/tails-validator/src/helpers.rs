use crate::types::{PathSegment, ValidationIssue};

pub fn type_err(expected: &str, value: &serde_json::Value, custom_error: Option<&str>) -> ValidationIssue {
    issue_custom("invalid_type", custom_error.unwrap_or(&format!("Expected {}", expected)), expected, &type_name(value))
}

pub fn range_err(code: &str, message: &str, expected: &str, received: &str) -> ValidationIssue {
    ValidationIssue { code: code.to_string(), message: message.to_string(), path: None, expected: Some(expected.to_string()), received: Some(received.to_string()) }
}

pub fn issue(code: &str, message: &str, expected: &str, received: &str) -> ValidationIssue {
    ValidationIssue { code: code.to_string(), message: message.to_string(), path: None, expected: Some(expected.to_string()), received: Some(received.to_string()) }
}

pub fn issue_custom(code: &str, message: &str, expected: &str, received: &str) -> ValidationIssue {
    ValidationIssue { code: code.to_string(), message: message.to_string(), path: None, expected: Some(expected.to_string()), received: Some(received.to_string()) }
}

pub fn path_issue(code: &str, message: &str, expected: &str, received: &str, key: &str) -> ValidationIssue {
    ValidationIssue { code: code.to_string(), message: message.to_string(), path: Some(vec![PathSegment { key: key.to_string() }]), expected: Some(expected.to_string()), received: Some(received.to_string()) }
}

pub fn type_name(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }.to_string()
}

pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len { s.to_string() } else { format!("{}...", &s[..max_len]) }
}

pub fn format_validation_error(issues: &[ValidationIssue]) -> String {
    issues.iter().map(|issue| {
        let path = issue.path.as_ref().map(|p| p.iter().map(|seg| seg.key.as_str()).collect::<Vec<_>>().join(".")).unwrap_or_else(|| "(root)".to_string());
        let mut line = format!("[{}] {}", path, issue.message);
        if let (Some(exp), Some(rec)) = (&issue.expected, &issue.received) {
            if !exp.is_empty() && !rec.is_empty() { line.push_str(&format!(" (expected: {}, received: {})", exp, rec)); }
        }
        line
    }).collect::<Vec<_>>().join("\n")
}
