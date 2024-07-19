use std::{
    collections::HashMap
    ,
    path::Path,
    sync::Arc,
};

use async_trait::async_trait;
use tokio::fs::read_to_string;

use crate::db::mem::Data;
use crate::Error;

pub type TemplatePool = Data<Box<str>, Result<Arc<str>, String>>;

#[async_trait]
pub trait GetTemplate {
    async fn get_template(
        &self,
        template_name: &str,
    ) -> Result<Arc<str>, String>;
}

#[async_trait]
impl GetTemplate for TemplatePool {
    async fn get_template(
        &self,
        template_name: &str,
    ) -> Result<Arc<str>, String> {
        let temp = self.get(&Box::from(template_name)).await.map_err(|e| e.to_string().into_boxed_str())?.ok_or("Template not found")?;
        match temp { 
            Ok(t) => Ok(t),
            Err(e) => Err(e),
        }
    }
}

pub async fn read_template(
    is_component: bool,
    template: &str,
    theme_dir: &Path,
    theme: &str,
) -> Result<Arc<str>, String> {
    let template = match is_component {
        true => format!("components/{}", template),
        false => template.to_owned(),
    };
    let theme = read_to_string(
        theme_dir
            .join(theme)
            .join("templates")
            .join(template)
            .with_extension("hbs"),
    )
    .await
    .map_err(|e| e.to_string());

    match theme {
        Ok(theme) => Ok(Arc::from(theme)),
        Err(e) => Err(e),
    }
}

pub async fn load_all_templates(
    theme_dir: &Path,
    theme: &str,
) -> Result<
    HashMap<Box<str>, Result<Arc<str>, String>>,
    Error,
> {
    let templates = vec![
        ("layout", true),
        ("navbar", true),
        ("overlay", true),
        ("blog", true),
        ("404", true),
        ("default", false),
    ];

    let mut template_pool = HashMap::new();

    for (name, is_component) in templates {
        let content = read_template(is_component, name, theme_dir, theme).await;
        template_pool.insert(name.to_string().into_boxed_str(), content);
    }

    Ok(template_pool)
}
