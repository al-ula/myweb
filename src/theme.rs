use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Theme {
    name: String,
    version: String,
    author: String,
    static_path: PathBuf,
    templates_path: PathBuf,
    templates: Templates,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Templates {
    components_path: PathBuf,
    templates: Vec<Template>,
    components: Vec<Template>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Template {
    name: String,
    path: PathBuf,
}
