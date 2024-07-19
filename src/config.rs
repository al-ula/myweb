use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub admin_port: Arc<u16>,
    pub theme: Arc<str>,
    pub theme_dir: Arc<Path>,
    pub db_dir: Arc<Path>,
}

impl Default for Config {
    fn default() -> Config {
        use std::env;
        let home_dir: PathBuf = if cfg!(target_os = "windows") {
            env::var("USERPROFILE").or_else(|_| {
                let drive = env::var("HOMEDRIVE").expect("HOMEDRIVE environment variable not set");
                let path = env::var("HOMEPATH").expect("HOMEPATH environment variable not set");
                Ok(format!("{}{}", drive, path))
            })
        } else {
            env::var("HOME")
        }
        .expect("Home directory environment variable not set")
        .into();

        let theme_dir: Arc<Path> = home_dir.join("my_web/theme").into();
        let db_dir: Arc<Path> = home_dir.join("my_web/db").into();
        if cfg!(debug_assertions) {
            return Config {
                admin_port: 8001.into(),
                theme: "default".into(),
                theme_dir: Path::new("theme").into(),
                db_dir: Path::new("db").into(),
            };
        }
        Config {
            admin_port: 8001.into(),
            theme: "default".into(),
            theme_dir,
            db_dir,
        }
    }
}
