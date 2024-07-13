mod account;
mod config;
mod json;
mod page;
mod post;
mod public;
mod string;
mod template;
mod theme;

use config::Config;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use page::PageCache;
use post::{Markdown, PreviewArticle};
use public::{blog, index, not_found, pages};
use rocket::{fairing::AdHoc, routes};
use std::collections::HashMap;
use std::sync::Arc;
use string::*;
use template::load_all_templates;
use tokio::{fs::read_to_string, sync::RwLock};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config/server.toml").nested())
        .merge(Env::prefixed("MY_WEB_").global())
        .select(Profile::from_env_or("MY_WEB_PROFILE", "default"));

    let theme_dir = &figment
        .extract::<Config>()
        .expect("Failed to extract config")
        .theme_dir;

    let theme = &figment
        .extract::<Config>()
        .expect("Failed to extract config")
        .theme;

    let template = match load_all_templates(theme_dir, theme).await {
        Ok(templates) => Arc::new(RwLock::new(templates)),
        Err(e) => {
            eprintln!("Failed to load templates: {}", e);
            std::process::exit(1);
        }
    };

    let page_cache: PageCache = Arc::new(RwLock::new(HashMap::new()));

    if cfg!(debug_assertions) {
        let markdown = Markdown::from(
            read_to_string("articles/blog/ant_dilemma.md")
                .await
                .unwrap(),
        );
        let article_prev = &markdown.preview().await.unwrap();
        let theme =
            match theme::Theme::read(&theme_dir.join(theme).join("meta").with_extension("toml"))
                .await
            {
                Ok(theme) => theme,
                Err(e) => {
                    eprintln!("Failed to read theme: {}", e);
                    std::process::exit(1);
                }
            };
        println!("Preview: {:#?}", article_prev);
        // println!("Templates: {:#?}", template.read().await);
        println!("Theme:\n{:#?}", theme);
    }

    let _rocket = rocket::custom(&figment)
        .attach(AdHoc::config::<Config>())
        .manage(template)
        .manage(page_cache)
        .mount("/", routes![index, pages, blog, not_found])
        .mount(
            "/static",
            rocket::fs::FileServer::from(theme_dir.join(theme).join("static")),
        )
        .launch()
        .await?;
    Ok(())
}
