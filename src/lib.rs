use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase", deny_unknown_fields)]
pub enum Predicate {
    Starts {
        value: String,
        path: String,
        #[serde(default)]
        ignore_case: bool,
    },

    Ends {
        value: String,
        path: String,
        #[serde(default)]
        ignore_case: bool,
    },

    Contains {
        value: Value,
        path: String,
        #[serde(default)]
        ignore_case: bool,
    },

    Defined {
        path: String,
    },

    In {
        path: String,
        value: Vec<Value>,
    },

    Less {
        path: String,
        value: f64,
    },

    Matches {
        path: String,
        value: String,
        #[serde(default)]
        ignore_case: bool,
    },

    More {
        path: String,
        value: f64,
    },

    Test {
        value: Value,
        path: String,
    },

    Type {
        value: Value,
        path: String,
    },

    Undefined {
        path: String,
    },

    And {
        path: Option<String>,
        apply: Vec<Predicate>,
    },

    Not {
        path: Option<String>,
        apply: Vec<Predicate>,
    },

    Or {
        path: Option<String>,
        apply: Vec<Predicate>,
    },
}

impl Predicate {
    fn str_representation(val: &Value, ignore_case: bool) -> Option<String> {
        match val {
            Value::String(s) => Some(if ignore_case {
                s.to_lowercase()
            } else {
                s.to_string()
            }),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        }
    }

    pub fn test(&self, input: &Value) -> bool {
        match self {
            Predicate::Starts {
                value,
                path,
                ignore_case,
            } => {
                if let Some(val) = input
                    .pointer(path)
                    .and_then(|v| Self::str_representation(v, *ignore_case))
                {
                    val.starts_with(value)
                } else {
                    false
                }
            }

            Predicate::Contains {
                value,
                path,
                ignore_case,
            } => {
                match (
                    input
                        .pointer(path)
                        .and_then(|v| Self::str_representation(v, *ignore_case)),
                    value.as_str(),
                ) {
                    (Some(val), Some(substr)) => val.contains(substr),
                    _ => false,
                }
            }

            Predicate::Ends {
                value,
                path,
                ignore_case,
            } => {
                if let Some(val) = input
                    .pointer(path)
                    .and_then(|v| Self::str_representation(v, *ignore_case))
                {
                    val.ends_with(value)
                } else {
                    false
                }
            }

            Predicate::Defined { path } => input.pointer(path).is_some(),

            Predicate::In { path, value } => input
                .pointer(path)
                .map(|val| value.iter().any(|v| v == val))
                .unwrap_or_default(),

            Predicate::Less { path, value } => input
                .pointer(path)
                .map(|v| v.as_number().and_then(|n| n.as_f64()) < Some(*value))
                .unwrap_or_default(),

            Predicate::Matches {
                path,
                value,
                ignore_case,
            } => {
                if let (Some(val), Some(regex)) = (
                    input.pointer(path).and_then(|v| v.as_str()),
                    RegexBuilder::new(value)
                        .case_insensitive(*ignore_case)
                        .build()
                        .ok(),
                ) {
                    regex.is_match(val)
                } else {
                    false
                }
            }

            Predicate::More { path, value } => input
                .pointer(path)
                .map(|v| v.as_number().and_then(|n| n.as_f64()) > Some(*value))
                .unwrap_or_default(),

            Predicate::Test { value, path } => input.pointer(path) == Some(value),

            Predicate::Type { value, path } => match input.pointer(path) {
                Some(Value::Number(_)) => value == "number",
                Some(Value::String(_)) => value == "string",
                Some(Value::Bool(_)) => value == "boolean",
                Some(Value::Object(_)) => value == "object",
                Some(Value::Array(_)) => value == "array",
                Some(Value::Null) => value == "null",
                None => value == "undefined",
            },

            Predicate::Undefined { path } => input.pointer(path).is_none(),

            Predicate::And { path, apply } => {
                if apply.is_empty() {
                    false
                } else {
                    apply.iter().all(|pred| {
                        if let Some(val) = input.pointer(path.as_deref().unwrap_or("")) {
                            pred.test(val)
                        } else {
                            false
                        }
                    })
                }
            }
            Predicate::Not { path, apply } => !apply.iter().all(|pred| {
                if let Some(val) = input.pointer(path.as_deref().unwrap_or("")) {
                    pred.test(val)
                } else {
                    false
                }
            }),

            Predicate::Or { path, apply } => apply.iter().any(|pred| {
                if let Some(val) = input.pointer(path.as_deref().unwrap_or("")) {
                    pred.test(val)
                } else {
                    false
                }
            }),
        }
    }
}
