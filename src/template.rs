use async_trait::async_trait;
use std::{
    collections::HashMap,
    error::Error,
    io::{Error as IoError, ErrorKind},
    sync::Arc,
};

use tokio::sync::RwLock;

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
