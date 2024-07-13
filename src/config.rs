use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub theme: String,
    pub theme_dir: PathBuf,
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
        if cfg!(debug_assertions) {
            return Config {
                theme: "default".to_string(),
                theme_dir: String::from("theme").into(),
            };
        }
        Config {
            theme: "default".to_string(),
            theme_dir,
        }
    }
}
