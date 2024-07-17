use std::{collections::HashMap, error::Error, sync::Arc};

use chrono::Duration;
use handlebars::Handlebars;
use rocket::State;
use serde_json::{Map, Value};
use tokio::{sync::RwLock, time::Instant};

pub type PageCache = Arc<RwLock<HashMap<String, (String, Instant)>>>;

use crate::{
    post::Html,
    template::{GetTemplate, TemplatePool},
};

pub fn make_data(data_list: &[(String, Value)]) -> Map<String, Value> {
    let data_list = data_list.to_owned();
    let mut data = Map::new();
    for datum in data_list {
        data.insert(datum.0, datum.1);
    }
    data
}

pub async fn render(
    page_template: &str,
    template_pool: &State<TemplatePool>,
    template_list: &[(&str, &str)],
    data: Map<String, Value>,
) -> Result<Html, Box<dyn Error + Send + Sync>> {
    let mut handlebars = Handlebars::new();
    for t in template_list.iter() {
        handlebars.register_template_string(t.0, template_pool.get_template(t.1).await?)?;
    }
    let hb = Html::new(handlebars.render(page_template, &data)?).minify()?;
    Ok(hb)
}

pub async fn get_or_render_page(
    page_template: &str,
    template_pool: &State<TemplatePool>,
    template_list: &[(&str, &str)],
    data: Map<String, Value>,
    page_cache: &State<PageCache>,
    cache_duration: Duration,
    cache_id: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    if cfg!(debug_assertions) {
        let generated_page = render(page_template, template_pool, template_list, data)
            .await?
            .minify()?
            .to_string();
        Ok(generated_page)
    } else {
        // Try to get the page from the cache
        {
            let cache = page_cache.read().await;
            if let Some((page, timestamp)) = cache.get(cache_id) {
                if timestamp.elapsed() < cache_duration.to_std()? {
                    return Ok(page.clone());
                }
            }
        }

        // If not in cache or expired, generate the page
        let generated_page = render(page_template, template_pool, template_list, data)
            .await?
            .minify()?
            .to_string();

        // Store the generated page in the cache
        {
            let mut cache = page_cache.write().await;
            cache.insert(
                cache_id.to_string(),
                (generated_page.clone(), Instant::now()),
            );
        }
        Ok(generated_page)
    }
}
