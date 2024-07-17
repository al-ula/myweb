use std::path::{Path, PathBuf};

use rocket::{
    fs::NamedFile,
    get,
    response::{status::NotFound, Redirect},
};

#[get("/assets/<file..>")]
pub async fn admin_assets(file: PathBuf) -> Result<NamedFile, NotFound<Redirect>> {
    let base = Path::new("admin/dist");
    match NamedFile::open(base.join("assets").join(file)).await {
        Ok(nf) => Ok(nf),
        Err(_e) => Err(NotFound(Redirect::to("admin/error"))),
    }
}

#[get("/<page>")]
pub async fn admin_page(page: &str) -> NamedFile {
    let _path = page;
    NamedFile::open(Path::new("admin/dist").join("index.html"))
        .await
        .unwrap()
}

#[get("/")]
pub async fn admin_index() -> NamedFile {
    NamedFile::open(Path::new("admin/dist").join("index.html"))
        .await
        .unwrap()
}
