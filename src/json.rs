use std::fmt::Display;

use serde_json::Value;

#[derive(Default, Clone, Debug)]
pub struct Json(Value);
impl Json {
    pub fn new(content: Value) -> Json {
        Json(content)
    }
}

impl From<Json> for Value {
    fn from(val: Json) -> Self {
        val.0
    }
}

impl Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<Value> for Json {
    fn from(content: Value) -> Self {
        Json(content)
    }
}

impl TryFrom<&str> for Json {
    type Error = serde_json::Error;
    fn try_from(content: &str) -> Result<Self, Self::Error> {
        let value = serde_json::from_str(content);
        match value {
            Ok(value) => Ok(Json(value)),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<String> for Json {
    type Error = serde_json::Error;
    fn try_from(content: String) -> Result<Self, Self::Error> {
        let value = serde_json::from_str(&content);
        match value {
            Ok(value) => Ok(Json(value)),
            Err(e) => Err(e),
        }
    }
}

impl TryFrom<Json> for String {
    type Error = serde_json::Error;
    fn try_from(json: Json) -> Result<Self, Self::Error> {
        let content = serde_json::to_string(&json.0)?;
        Ok(content)
    }
}
