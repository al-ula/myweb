mod post;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use handlebars::{to_json, Handlebars};
use post::Html;
use rocket::{
    fairing::AdHoc,
    get,
    response::{content::RawHtml, Redirect},
    routes, State,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config/server.toml").nested())
        .merge(Env::prefixed("MY_WEB_").global())
        .select(Profile::from_env_or("MY_WEB_PROFILE", "default"));
    let _rocket = rocket::custom(&figment)
        .attach(AdHoc::config::<Config>())
        .mount("/", routes![index, page])
        .mount(
            "/static",
            rocket::fs::FileServer::from(
                &figment
                    .extract::<Config>()
                    .expect("Failed to extract config")
                    .static_path,
            ),
        )
        .launch()
        .await?;
    Ok(())
}

#[derive(Serialize, Debug)]
struct Menu {
    name: String,
    url: String,
}

fn make_data(menus: Vec<Menu>) -> Map<String, Value> {
    let mut data = Map::new();
    data.insert("menus".to_string(), to_json(menus));
    data.insert("title".to_string(), to_json("ISAALULA"));
    data.insert("default_theme".to_string(), to_json("mocha"));
    data.insert("secondary_theme".to_string(), to_json("latte"));
    data.insert("parent".to_string(), to_json("layout"));
    data
}

async fn make_page(
    template_path: &str,
    page_name: &str,
) -> Result<post::Html, Box<dyn std::error::Error>> {
    let menus: Vec<Menu> = vec![
        Menu {
            name: "Blog".to_string(),
            url: "/blog".to_string(),
        },
        Menu {
            name: "Projects".to_string(),
            url: "/projects".to_string(),
        },
        Menu {
            name: "About".to_string(),
            url: "/about".to_string(),
        },
    ];
    let page_template = format!("./templates/{}.hbs", page_name);
    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_file("navbar", format!("{}/components/navbar.hbs", template_path))?;
    handlebars.register_template_file(
        "overlay",
        format!("{}/components/overlay.hbs", template_path),
    )?;
    handlebars.register_template_file(
        "layout",
        format!("{}/components/main_layout.hbs", template_path),
    )?;
    handlebars.register_template_file(page_name, page_template)?;
    let data = make_data(menus);
    let hb = Html::new(handlebars.render(page_name, &data)?).minify()?;
    Ok(hb)
}

#[get("/")]
async fn index(template_path: &State<Config>) -> RawHtml<String> {
    let template_path = &template_path.templates_path;
    let html = make_page(template_path, "index").await.unwrap();
    RawHtml(html.to_string())
}

#[get("/<page>")]
async fn page(page: String, template_path: &State<Config>) -> Result<RawHtml<String>, Redirect> {
    let template_path = &template_path.templates_path;
    match page.as_str() {
        "index" => Err(Redirect::to("/")),
        "blog" => Ok(RawHtml(
            make_page(template_path, "blog")
                .await
                .unwrap_or(Html::new("".to_string()))
                .to_string(),
        )),
        "projects" => Ok(RawHtml(
            make_page(template_path, "projects")
                .await
                .unwrap_or(Html::new("".to_string()))
                .to_string(),
        )),
        "about" => Ok(RawHtml(
            make_page(template_path, "about")
                .await
                .unwrap_or(Html::new("".to_string()))
                .to_string(),
        )),
        _ => Ok(RawHtml(
            Html::new("404 page not found".to_string()).to_string(),
        )),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    static_path: String,
    templates_path: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            static_path: String::from("static"),
            templates_path: String::from("templates"),
        }
    }
}
