mod post;

use std::path::PathBuf;

use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use handlebars::{to_json, Handlebars};
use post::Html;
use rocket::{
    fairing::AdHoc,
    get,
    response::{content::RawHtml, status::NotFound, Redirect},
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

fn make_page(
    config: &State<Config>,
    page_name: &str,
    data: Map<String, Value>,
) -> Result<post::Html, Box<dyn std::error::Error>> {
    let template_path = match config
        .theme_dir
        .to_owned()
        .join(&config.theme)
        .join("templates")
        .to_str()
    {
        Some(t) => t.to_owned(),
        None => return Err("Failed to get template path".into()),
    };
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
async fn index(config: &State<Config>) -> Result<RawHtml<String>, NotFound<String>> {
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
    let html = make_page(config, "index", data);
    match html {
        Ok(html) => Ok(RawHtml(html.to_string())),
        Err(e) => Err(NotFound(e.to_string())),
    }
}

#[get("/<page>")]
async fn page(
    page: &str,
    config: &State<Config>,
) -> Result<Result<RawHtml<String>, Redirect>, NotFound<String>> {
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
    let page_title: [&str; 3] = ["blog", "projects", "about"];
    match page {
        "index" => Ok(Err(Redirect::to("/"))),
        p if page_title.contains(&p) => {
            let html = make_page(config, p, data);
            match html {
                Ok(html) => Ok(Ok(RawHtml(html.to_string()))),
                Err(e) => Err(NotFound(e.to_string())),
            }
        }
        _ => Err(NotFound("Page not found".to_string())),
    }
}

#[get("/blog/<blog_post>")]
async fn blog(
    blog_post: &str,
    config: &State<Config>,
) -> Result<RawHtml<String>, NotFound<String>> {
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

    let html = match post::Markdown::new(blog_content).to_html(post::MarkdownType::Common) {
        Ok(h) => h.to_string(),
        Err(e) => return Err(NotFound(e.to_string())),
    };
    let mut data = make_data(menus, &blog_post.snake_to_title_case());
    data.insert("blog_post".to_string(), to_json(html));
    let html = make_page(config, "blog_post", data);
    match html {
        Ok(html) => Ok(RawHtml(html.to_string())),
        Err(e) => Err(NotFound(e.to_string())),
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    theme: String,
    theme_dir: PathBuf,
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

        Config {
            theme: "default".to_string(),
            theme_dir,
        }
    }
}

trait SnakeToTitleCase {
    fn snake_to_title_case(&self) -> String;
}

impl SnakeToTitleCase for &str {
    fn snake_to_title_case(&self) -> String {
        fn capitalize_first_letter(s: &str) -> String {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }

        self.split('_')
            .map(capitalize_first_letter)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl SnakeToTitleCase for String {
    fn snake_to_title_case(&self) -> String {
        self.as_str().snake_to_title_case()
    }
}
