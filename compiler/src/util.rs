use human_parser::{ConstraintLevel, Value};

pub fn level_str(level: ConstraintLevel) -> &'static str {
    match level {
        ConstraintLevel::Never => "NEVER",
        ConstraintLevel::Must => "MUST",
        ConstraintLevel::Should => "SHOULD",
        ConstraintLevel::Avoid => "AVOID",
        ConstraintLevel::May => "MAY",
    }
}

/// Bare value: unquoted strings. For prompt and txt output.
pub fn format_value_bare(val: &Value) -> String {
    match val {
        Value::Str(s) => s.clone(),
        Value::Number(n) => {
            if *n == (*n as i64) as f64 {
                format!("{}", *n as i64)
            } else {
                format!("{n}")
            }
        }
        Value::Bool(b) => format!("{b}"),
        Value::Path(p) => p.clone(),
    }
}

/// HMN value: quoted strings with \" and \\ escaping. For hmn output.
pub fn format_value_hmn(val: &Value) -> String {
    match val {
        Value::Str(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{escaped}\"")
        }
        Value::Number(n) => {
            if *n == (*n as i64) as f64 {
                format!("{}", *n as i64)
            } else {
                format!("{n}")
            }
        }
        Value::Bool(b) => format!("{b}"),
        Value::Path(p) => p.clone(),
    }
}

/// JSON value for serde serialization. For ir.rs.
pub fn value_to_json(val: &Value) -> serde_json::Value {
    match val {
        Value::Str(s) => serde_json::Value::String(s.clone()),
        Value::Number(n) => serde_json::json!(n),
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Path(p) => serde_json::Value::String(p.clone()),
    }
}
