use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf};
use tokio::fs::read_to_string;
use toml::de;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Theme {
    pub name: String,
    pub version: String,
    pub author: String,
    pub static_path: PathBuf,
    pub templates_path: PathBuf,
    pub templates: Templates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Templates {
    pub components_path: PathBuf,
    pub templates: Vec<Template>,
    pub components: Vec<Template>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    pub name: String,
    pub path: PathBuf,
    pub components: Option<Vec<String>>,
    pub override_components: Option<Vec<String>>,
    pub variables: Option<Vec<(Variable, String)>>,
    pub override_variables: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Variable {
    Bool,
    String,
    Value,
    Raw,
}

impl Theme {
    pub async fn read(path: &PathBuf) -> Result<Theme, Box<dyn Error + Send + Sync>> {
        let theme = read_to_string(path).await?;
        de::from_str(&theme).map_err(|e| e.into())
    }
}
