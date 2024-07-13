use async_trait::async_trait;
use std::{
    collections::HashMap,
    error::Error,
    io::{Error as IoError, ErrorKind},
    path::Path,
    sync::Arc,
};

use tokio::{fs::read_to_string, sync::RwLock};

pub type TemplatePool = Arc<RwLock<HashMap<String, Result<String, Box<dyn Error + Send + Sync>>>>>;

#[async_trait]
pub trait GetTemplate {
    async fn get_template(
        &self,
        template_name: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl GetTemplate for TemplatePool {
    async fn get_template(
        &self,
        template_name: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let templates = self.read().await;
        match templates.get(template_name) {
            Some(Ok(template)) => Ok(template.to_string()),
            Some(Err(e)) => Err(IoError::new(
                ErrorKind::Other,
                format!("Failed to read template '{}': {}", template_name, e),
            )
            .into()),
            None => Err(IoError::new(
                ErrorKind::NotFound,
                format!("Template '{}' not found", template_name),
            )
            .into()),
        }
    }
}

pub async fn read_template(
    is_component: bool,
    template: &str,
    theme_dir: &Path,
    theme: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let template = match is_component {
        true => format!("components/{}", template),
        false => template.to_owned(),
    };
    read_to_string(
        theme_dir
            .join(theme)
            .join("templates")
            .join(template)
            .with_extension("hbs"),
    )
    .await
    .map_err(|e| e.into())
}

pub async fn load_all_templates(
    theme_dir: &Path,
    theme: &str,
) -> Result<
    HashMap<String, Result<String, Box<dyn Error + Send + Sync>>>,
    Box<dyn Error + Send + Sync>,
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
        template_pool.insert(name.to_string(), content);
    }

    Ok(template_pool)
}
