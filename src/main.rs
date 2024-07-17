mod account;
mod admin;
mod config;
mod db;
mod json;
mod post;
mod public;
mod render;
mod string;
mod template;
mod theme;

use admin::{admin_assets, admin_index, admin_page};
use config::Config;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use post::{Markdown, PreviewArticle};
use public::{blog, index, not_found, pages, static_files};
use render::PageCache;
use rocket::{fairing::AdHoc, routes, Ignite, Build};
use std::collections::HashMap;
use std::error;
use std::sync::Arc;
use ammonia::url::quirks::set_port;
use string::*;
use template::load_all_templates;
use tokio::{fs::read_to_string, sync::RwLock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    init().await;

    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config/server.toml").nested())
        .merge(Env::prefixed("MY_WEB_").global())
        .select(Profile::from_env_or("MY_WEB_PROFILE", "default"));
    
    let public = rocket(&figment).await.map_err(|e| e.to_string())?;
    let admin = rocket_admin(&figment).await.map_err(|e| e.to_string())?;
    
    let public_task = tokio::task::spawn(async move {
        public
            .launch()
            .await
            .expect("Failed to ignite public server")
    });
    let admin_task = tokio::task::spawn(async move {
        admin
            .launch()
            .await
            .expect("Failed to ignite admin server")
    });
    let _tasks = tokio::join!(public_task, admin_task);
    end().await;
    Ok(())
}

async fn init() {
    //     TODO
}

async fn rocket(figment: &Figment) -> Result<rocket::Rocket<Build>, rocket::Error> {
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

    let rocket = rocket::custom(&figment)
        .attach(AdHoc::config::<Config>())
        .manage(template)
        .manage(page_cache)
        .mount("/", routes![index, static_files, blog, pages, not_found]);

    Ok(rocket)
}

async fn rocket_admin(figment: &Figment) -> Result<rocket::Rocket<Build>, rocket::Error> {
    
    let port = &figment.extract::<Config>().expect("Failed to extract config").admin_port;

    let figment = figment.clone().merge(("port", port));

    let rocket = rocket::custom(&figment)
        .mount("/", routes![admin_index, admin_assets, admin_page]);

    Ok(rocket)
}

async fn end() {
    //     TODO
}
