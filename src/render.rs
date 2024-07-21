use std::sync::Arc;
use crate::Error;
use chrono::Duration;
use handlebars::Handlebars;
use rocket::State;
use serde_json::{Map, Value};
use tokio::time::Instant;

use crate::{
    post::Html,
    template::{GetTemplate, TemplatePool},
};
use crate::db::mem::Data;

pub type PageCache = Data<String, (Arc<str>, Instant)>;

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
) -> Result<Html, Error> {
    let mut handlebars = Handlebars::new();
    for t in template_list.iter() {
        let template = template_pool.get_template(t.1).await?; 
        handlebars.register_template_string(t.0, template)?;
    }
    let hb = Html::new(handlebars.render(page_template, &data)?).minify()?;
    Ok(hb)
}

pub async fn render_page(
    page_template: &str,
    template_pool: &State<TemplatePool>,
    template_list: &[(&str, &str)],
    data: Map<String, Value>,
    page_cache: &State<PageCache>,
    cache_id: &str,
) -> Result<Arc<str>, Error> {
        let generated_page: Arc<str> = render(page_template, template_pool, template_list, data)
            .await?
            .minify()?.to_string().into();

        // Store the generated page in the cache
        {
            page_cache.delete(&cache_id.to_string()).await?;
            page_cache.insert(
                cache_id.to_string(),
                (generated_page.clone(), Instant::now()),
            ).await?;
        }
        Ok(generated_page)
}

pub async fn get_page(page_cache: &State<PageCache>, cache_duration: Duration, cache_id: &str) -> Result<Option<Arc<str>>, Error> {
    let cache = page_cache.get(&cache_id.to_string()).await?;
    if let Some((page, timestamp)) = cache {
        if timestamp.elapsed() < cache_duration.to_std()? {
            Ok(Some(page.clone()))
        } else {
            page_cache.delete(&cache_id.to_string()).await?;
            Ok(Some(page.clone()))
        }
    } else { 
        Ok(None)
    }
}