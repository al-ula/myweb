use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub admin_port: u16,
    pub theme: String,
    pub theme_dir: PathBuf,
    pub db_dir: PathBuf,
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

        let theme_dir = home_dir.join("my_web/theme");
        let db_dir = home_dir.join("my_web/db");
        if cfg!(debug_assertions) {
            return Config {
                admin_port: 8001,
                theme: "default".to_string(),
                theme_dir: String::from("theme").into(),
                db_dir: String::from("db").into(),
            };
        }
        Config {
            admin_port: 8001,
            theme: "default".to_string(),
            theme_dir,
            db_dir
        }
    }
}
