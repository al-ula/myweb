use derive_more::{Display, From};
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, From, Display)]
pub enum Error {
    #[from]
    Fmt(std::fmt::Error),
    #[from]
    Io(std::io::Error),
    #[from]
    Rocket(rocket::Error),
    #[from]
    Json(serde_json::Error),
    #[from]
    String(String),
}