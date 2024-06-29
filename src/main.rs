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
use tokio::fs;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config/server.toml").nested())
        .merge(Env::prefixed("MY_WEB_").global())
        .select(Profile::from_env_or("MY_WEB_PROFILE", "default"));

    let theme = &figment
        .extract::<Config>()
        .expect("Failed to extract config")
        .theme;
    println!("Theme: {}", theme);
    let _rocket = rocket::custom(&figment)
        .attach(AdHoc::config::<Config>())
        .mount("/", routes![index, page, blog])
        .mount(
            "/static",
            rocket::fs::FileServer::from(format!("theme/{}/static", theme).as_str()),
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

fn make_data(menus: Vec<Menu>, title: &str) -> Map<String, Value> {
    let mut data = Map::new();
    data.insert("menus".to_string(), to_json(menus));
    data.insert("title".to_string(), to_json(title));
    data.insert("default_theme".to_string(), to_json("mocha"));
    data.insert("secondary_theme".to_string(), to_json("latte"));
    data.insert("parent".to_string(), to_json("layout"));
    data
}

async fn make_page(
    template_path: &str,
    page_name: &str,
    data: Map<String, Value>,
) -> Result<post::Html, Box<dyn std::error::Error>> {
    let page_template = format!("{}/{}.hbs", template_path, page_name);
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
    let hb = Html::new(handlebars.render(page_name, &data)?).minify()?;
    Ok(hb)
}

#[get("/")]
async fn index(config: &State<Config>) -> RawHtml<String> {
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
    let data = make_data(menus, "ISAALULA");
    let template_path = format!("theme/{}/templates", &config.theme);
    println!("Template path: {}", template_path);
    let html = make_page(&template_path, "index", data).await.unwrap();
    RawHtml(html.to_string())
}

#[get("/<page>")]
async fn page(page: String, config: &State<Config>) -> Result<RawHtml<String>, Redirect> {
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
    let data = make_data(menus, &page.to_uppercase());
    let template_path = format!("theme/{}/templates", &config.theme);
    let page_title: [&str; 3] = ["blog", "projects", "about"];
    if let Some(title) = page_title.iter().find(|&&t| page == t) {
        let html = RawHtml(
            make_page(&template_path, title, data)
                .await
                .unwrap_or_default()
                .to_string(),
        );
        return Ok(html);
    }
    match page.as_str() {
        "index" => Err(Redirect::to("/")),
        _ => Ok(RawHtml(
            Html::new("404 page not found".to_string()).to_string(),
        )),
    }
}

#[get("/blog/<blog_post>")]
async fn blog(blog_post: String, config: &State<Config>) -> Result<RawHtml<String>, Redirect> {
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
    let blog_content = fs::read_to_string(format!("articles/blog/{}.md", blog_post))
        .await
        .expect("Failed to read blog content");
    let html = post::Markdown::new(blog_content)
        .to_html(post::MarkdownType::Common)
        .unwrap()
        .to_string();
    let mut data = make_data(menus, "Post");
    data.insert("blog_post".to_string(), to_json(&html));
    let template_path = format!("theme/{}/templates", &config.theme);
    let html = make_page(&template_path, "blog_post", data)
        .await
        .unwrap_or_default()
        .to_string();
    Ok(RawHtml(html))
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    theme: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            theme: "default".to_string(),
        }
    }
}
