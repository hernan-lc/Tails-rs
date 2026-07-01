mod engine;
mod formats;
mod helpers;
mod types;

use tails_native_macros::{tails_function, tails_module};

#[tails_module(name = "tails-validator")]
mod validator {
    use super::*;
    use serde_json::Value;
    use tails_abi::{FromNativeValue, NativeValue, ToNativeValue};

    // ========================================================================
    // Core validation
    // ========================================================================

    #[tails_function]
    pub fn validate(schema: NativeValue, value: NativeValue) -> String {
        let schema_val: Value = match Value::from_native_value(schema) {
            Ok(v) => v,
            Err(e) => {
                return serde_json::to_string(&types::ValidationErr {
                    success: false,
                    error: types::ValidationError {
                        issues: vec![helpers::issue(
                            "invalid_schema",
                            &format!("Invalid schema: {}", e),
                            "valid schema definition",
                            "NativeValue conversion error",
                        )],
                    },
                })
                .unwrap_or_default()
            }
        };
        let value_val: Value = match Value::from_native_value(value) {
            Ok(v) => v,
            Err(e) => {
                return serde_json::to_string(&types::ValidationErr {
                    success: false,
                    error: types::ValidationError {
                        issues: vec![helpers::issue(
                            "invalid_value",
                            &format!("Invalid value: {}", e),
                            "valid JSON value",
                            "NativeValue conversion error",
                        )],
                    },
                })
                .unwrap_or_default()
            }
        };
        let schema_def: types::SchemaDef = match serde_json::from_value(schema_val.clone()) {
            Ok(s) => s,
            Err(e) => {
                return serde_json::to_string(&types::ValidationErr {
                    success: false,
                    error: types::ValidationError {
                        issues: vec![helpers::issue(
                            "invalid_schema",
                            &format!("Schema parse error: {} | input: {}", e, schema_val),
                            "valid schema definition",
                            "deserialization error",
                        )],
                    },
                })
                .unwrap_or_default()
            }
        };
        let result = match engine::validate_value(&schema_def, &value_val) {
            Ok(data) => serde_json::to_string(&types::ValidationOk {
                success: true,
                data,
            })
            .unwrap_or_default(),
            Err(issues) => serde_json::to_string(&types::ValidationErr {
                success: false,
                error: types::ValidationError { issues },
            })
            .unwrap_or_default(),
        };
        result
    }

    #[tails_function]
    pub fn format_error(error: NativeValue) -> String {
        let err_val: Value = Value::from_native_value(error).unwrap_or_default();
        let err: types::ValidationErr =
            serde_json::from_value(err_val).unwrap_or_else(|_| types::ValidationErr {
                success: false,
                error: types::ValidationError { issues: vec![] },
            });
        helpers::format_validation_error(&err.error.issues)
    }

    // ========================================================================
    // Base type builders
    // ========================================================================

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn z() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"any"})).unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn string() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"string"})).unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn number() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"number"})).unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn boolean() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"boolean"})).unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn nil() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"null"})).unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn any() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"any"})).unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn unknown() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"unknown"})).unwrap()
    }

    // ========================================================================
    // String validators
    // ========================================================================

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringMin(min: f64) -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","minLength":min as usize}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringMax(max: f64) -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","maxLength":max as usize}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringLength(len: f64) -> NativeValue {
        let v = len as usize;
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","minLength":v,"maxLength":v}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringPattern(pattern: String) -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","pattern":pattern}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringEmail() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","format":"email"}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringUrl() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","format":"url"}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringUuid() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","format":"uuid"}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringDatetime() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","format":"datetime"}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringIPv4() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","format":"ipv4"}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringIPv6() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","format":"ipv6"}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringPhone() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","format":"phone"}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn stringBase64() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"string","format":"base64"}),
        )
        .unwrap()
    }

    // ========================================================================
    // Number validators
    // ========================================================================

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn numberMin(min: f64) -> NativeValue {
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"number","min":min}))
            .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn numberMax(max: f64) -> NativeValue {
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"number","max":max}))
            .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn numberInt() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"number","integer":true}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn numberPositive() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"number","positive":true}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn numberNegative() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"number","negative":true}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn numberMultipleOf(n: f64) -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"number","multipleOf":n}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn numberFinite() -> NativeValue {
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"number","finite":true}),
        )
        .unwrap()
    }

    // ========================================================================
    // Array validators
    // ========================================================================

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn arrayMin(itemsSchema: NativeValue, min: f64) -> NativeValue {
        let items: Value = Value::from_native_value(itemsSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"array","items":items,"minLength":min as usize}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn arrayMax(itemsSchema: NativeValue, max: f64) -> NativeValue {
        let items: Value = Value::from_native_value(itemsSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"array","items":items,"maxLength":max as usize}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn arrayLength(itemsSchema: NativeValue, len: f64) -> NativeValue {
        let items: Value = Value::from_native_value(itemsSchema).unwrap_or_default();
        let v = len as usize;
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"array","items":items,"minLength":v,"maxLength":v}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn arrayUnique(itemsSchema: NativeValue) -> NativeValue {
        let items: Value = Value::from_native_value(itemsSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"array","items":items,"uniqueItems":true}),
        )
        .unwrap()
    }

    // ========================================================================
    // Composable validators
    // ========================================================================

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn optional(innerSchema: NativeValue) -> NativeValue {
        let inner: Value = Value::from_native_value(innerSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"optional","inner":inner}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn nullable(innerSchema: NativeValue) -> NativeValue {
        let inner: Value = Value::from_native_value(innerSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"nullable","inner":inner}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn transform(innerSchema: NativeValue, transformName: String) -> NativeValue {
        let inner: Value = Value::from_native_value(innerSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"transform","inner":inner,"transform":transformName}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn refine(innerSchema: NativeValue, message: String) -> NativeValue {
        let inner: Value = Value::from_native_value(innerSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"refine","inner":inner,"message":message}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn pipe(schemas: NativeValue) -> NativeValue {
        let schemas_val: Value = Value::from_native_value(schemas).unwrap_or_default();
        let schemas_arr = schemas_val.as_array().cloned().unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"pipeline","schemas":schemas_arr}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn preprocess(transformName: String, innerSchema: NativeValue) -> NativeValue {
        let inner: Value = Value::from_native_value(innerSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"preprocess","transform":transformName,"inner":inner}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn withDefault(innerSchema: NativeValue, defaultValue: NativeValue) -> NativeValue {
        let inner: Value = Value::from_native_value(innerSchema).unwrap_or_default();
        let default: Value = Value::from_native_value(defaultValue).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"default","inner":inner,"default":default}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn customError(innerSchema: NativeValue, message: String) -> NativeValue {
        let mut inner: Value = Value::from_native_value(innerSchema).unwrap_or_default();
        if let Some(obj) = inner.as_object_mut() {
            obj.insert(
                "customError".to_string(),
                serde_json::Value::String(message),
            );
        }
        <Value as ToNativeValue>::to_native_value(&inner).unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn lazy(id: String, innerSchema: NativeValue) -> NativeValue {
        let schema: Value = Value::from_native_value(innerSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"lazy","id":id,"schema":schema}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn literal(value: NativeValue) -> NativeValue {
        let val: Value = Value::from_native_value(value).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"literal","value":val}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn enumValues(values: NativeValue) -> NativeValue {
        let vals: Value = Value::from_native_value(values).unwrap_or_default();
        let arr = vals.as_array().cloned().unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"enum","values":arr}))
            .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn object(properties: NativeValue, required: NativeValue, strict: bool) -> NativeValue {
        let props_val: Value = Value::from_native_value(properties).unwrap_or_default();
        let props_map = props_val.as_object().cloned().unwrap_or_default();
        let req_val: Value = Value::from_native_value(required).unwrap_or_default();
        let req_arr: Vec<String> = req_val
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(&serde_json::json!({"type":"object","properties":props_map,"required":req_arr,"strict":strict})).unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn record(valuesSchema: NativeValue) -> NativeValue {
        let values: Value = Value::from_native_value(valuesSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"record","values":values}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn union(schemas: NativeValue) -> NativeValue {
        let schemas_val: Value = Value::from_native_value(schemas).unwrap_or_default();
        let schemas_arr = schemas_val.as_array().cloned().unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"union","schemas":schemas_arr}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn intersection(schemas: NativeValue) -> NativeValue {
        let schemas_val: Value = Value::from_native_value(schemas).unwrap_or_default();
        let schemas_arr = schemas_val.as_array().cloned().unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"intersection","schemas":schemas_arr}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn tuple(schemas: NativeValue) -> NativeValue {
        let schemas_val: Value = Value::from_native_value(schemas).unwrap_or_default();
        let schemas_arr = schemas_val.as_array().cloned().unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"tuple","schemas":schemas_arr}),
        )
        .unwrap()
    }

    #[tails_function]
    #[allow(non_snake_case)]
    pub fn coerce(target: String, innerSchema: NativeValue) -> NativeValue {
        let inner: Value = Value::from_native_value(innerSchema).unwrap_or_default();
        <Value as ToNativeValue>::to_native_value(
            &serde_json::json!({"type":"coerce","target":target,"inner":inner}),
        )
        .unwrap()
    }
}
