use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[repr(transparent)]
pub struct User(Value);

impl User {
    pub fn find_fields(&self) -> Vec<(String, String)> {
        Self::descend(String::new(), &self.0)
    }

    fn descend(path: String, value: &Value) -> Vec<(String, String)> {
        match value {
            Value::Null => vec![(path, String::from("null"))],
            Value::Bool(v) => vec![(path, v.to_string())],
            Value::Number(v) => vec![(path, v.to_string())],
            Value::String(v) => vec![(path, v.to_owned())],
            Value::Array(v) => v
                .iter()
                .enumerate()
                .flat_map(|(i, v)| Self::descend(format!("{path}[{i}]"), v))
                .collect(),
            Value::Object(v) => v
                .iter()
                .flat_map(|(k, v)| Self::descend(format!("{path}.{k}"), v))
                .collect(),
        }
    }
}
