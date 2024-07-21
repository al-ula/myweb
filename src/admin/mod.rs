use std::path::{Path, PathBuf};
use figment::Figment;
use rocket::{Build, fs::NamedFile, get, response::{status::NotFound, Redirect}, routes};
use rocket::http::Status;
use crate::config::Config;

pub async fn launch(figment: &Figment) -> Result<rocket::Rocket<Build>, rocket::Error> {
    let port = &figment
        .extract::<Config>()
        .expect("Failed to extract config")
        .admin_port;

    let figment = figment.clone().merge(("port", port));

    let rocket =
        rocket::custom(figment).mount("/", routes![admin_index, admin_assets, admin_page]);

    Ok(rocket)
}


#[get("/assets/<file..>")]
pub async fn admin_assets(file: PathBuf) -> Result<NamedFile, NotFound<Redirect>> {
    let base = Path::new("admin/dist");
    NamedFile::open(base.join("assets").join(file)).await.map_err(|_|NotFound(Redirect::to("admin/error")))
}

#[get("/<page>")]
pub async fn admin_page(page: &str) -> Result<NamedFile, status::Custom<&'static str>> {
    let _page = page;
    NamedFile::open(Path::new("admin/dist").join("index.html"))
        .await
        .map_err(|_| status::Custom(Status::InternalServerError, "Failed to open html file"))
}

#[get("/")]
pub async fn admin_index() -> Result<NamedFile, status::Custom<&'static str>> {
    NamedFile::open(Path::new("admin/dist").join("index.html"))
        .await
        .map_err(|_| status::Custom(Status::InternalServerError, "Failed to open html file"))
}

use rocket::response::status;