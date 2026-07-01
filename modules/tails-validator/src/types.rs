use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum SchemaDef {
    #[serde(rename = "string")]
    String {
        #[serde(default, rename = "minLength")]
        min_length: Option<usize>,
        #[serde(default, rename = "maxLength")]
        max_length: Option<usize>,
        #[serde(default)]
        pattern: Option<String>,
        #[serde(default)]
        format: Option<String>,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "number")]
    Number {
        #[serde(default)]
        min: Option<f64>,
        #[serde(default)]
        max: Option<f64>,
        #[serde(default)]
        integer: bool,
        #[serde(default)]
        positive: bool,
        #[serde(default)]
        negative: bool,
        #[serde(default, rename = "multipleOf")]
        multiple_of: Option<f64>,
        #[serde(default)]
        finite: bool,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "boolean")]
    Boolean {
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "null")]
    Null {
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "undefined")]
    Undefined,
    #[serde(rename = "any")]
    Any,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "literal")]
    Literal {
        value: serde_json::Value,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "enum")]
    Enum {
        values: Vec<serde_json::Value>,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "object")]
    Object {
        properties: HashMap<String, SchemaDef>,
        #[serde(default)]
        required: Vec<String>,
        #[serde(default)]
        strict: bool,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "array")]
    Array {
        items: Box<SchemaDef>,
        #[serde(default, rename = "minLength")]
        min_length: Option<usize>,
        #[serde(default, rename = "maxLength")]
        max_length: Option<usize>,
        #[serde(default, rename = "uniqueItems")]
        unique_items: bool,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "tuple")]
    Tuple {
        schemas: Vec<SchemaDef>,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "record")]
    Record {
        values: Box<SchemaDef>,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "union")]
    Union {
        schemas: Vec<SchemaDef>,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "intersection")]
    Intersection {
        schemas: Vec<SchemaDef>,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "optional")]
    Optional { inner: Box<SchemaDef> },
    #[serde(rename = "nullable")]
    Nullable { inner: Box<SchemaDef> },
    #[serde(rename = "default")]
    DefaultValue {
        inner: Box<SchemaDef>,
        default: serde_json::Value,
    },
    #[serde(rename = "coerce")]
    Coerce {
        target: String,
        inner: Box<SchemaDef>,
    },
    #[serde(rename = "refine")]
    Refine {
        inner: Box<SchemaDef>,
        #[serde(default)]
        message: Option<String>,
    },
    #[serde(rename = "transform")]
    Transform {
        inner: Box<SchemaDef>,
        transform: String,
    },
    #[serde(rename = "lazy")]
    Lazy {
        #[allow(dead_code)]
        id: String,
        schema: Box<SchemaDef>,
    },
    #[serde(rename = "pipeline")]
    Pipeline {
        schemas: Vec<SchemaDef>,
        #[serde(default, rename = "customError")]
        custom_error: Option<String>,
    },
    #[serde(rename = "preprocess")]
    Preprocess {
        transform: String,
        inner: Box<SchemaDef>,
    },
}

// ============================================================================
// Validation Result Types
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct ValidationOk {
    pub success: bool,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErr {
    pub success: bool,
    pub error: ValidationError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub issues: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<Vec<PathSegment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub received: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathSegment {
    pub key: String,
}
