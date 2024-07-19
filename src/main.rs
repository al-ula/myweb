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

use config::Config;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use std::error;
use string::*;
pub type Error = Box<dyn error::Error + Send + Sync>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config/server.toml").nested())
        .merge(Env::prefixed("MY_WEB_").global())
        .select(Profile::from_env_or("MY_WEB_PROFILE", "default"));

    let public = public::launch(&figment).await.map_err(|e| e.to_string())?;
    let admin = admin::launch(&figment).await.map_err(|e| e.to_string())?;

    let public_task = tokio::task::spawn(async move {
        public
            .launch()
            .await
            .expect("Failed to ignite public server")
    });
    let admin_task =
        tokio::task::spawn(
            async move { admin.launch().await.expect("Failed to ignite admin server") },
        );
    let _tasks = tokio::join!(public_task, admin_task);
    Ok(())
}
