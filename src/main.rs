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
mod error;
pub use error::{Result, Error};
use config::Config;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use string::*;

#[tokio::main]
async fn main() -> Result<()> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config/server.toml").nested())
        .merge(Env::prefixed("MY_WEB_").global())
        .select(Profile::from_env_or("MY_WEB_PROFILE", "default"));

    let public = public::launch(&figment).await?;
    let admin = admin::launch(&figment).await?;

    let public_task = tokio::task::spawn(async move {
        public
            .launch()
            .await
    });
    let admin_task =
        tokio::task::spawn(
            async move { admin.launch().await },
        );
    let _tasks = tokio::join!(public_task, admin_task);
    Ok(())
}
