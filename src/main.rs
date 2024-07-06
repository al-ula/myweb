mod post;

use async_trait::async_trait;
use chrono::Duration;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use handlebars::{to_json, Handlebars};
use post::{Html, Json};
use rocket::{
    fairing::AdHoc,
    get,
    response::{content::RawHtml, status::NotFound, Redirect},
    routes, State,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{
    collections::HashMap,
    error::Error,
    io::ErrorKind,
    path::{Path, PathBuf},
};
use std::{io::Error as IoError, sync::Arc};
use tokio::{fs::read_to_string, sync::RwLock, time::Instant};
type TemplatePool = Arc<RwLock<HashMap<String, Result<String, Box<dyn Error + Send + Sync>>>>>;
type PageCache = Arc<RwLock<HashMap<String, (String, Instant)>>>;
type Handle404<T> = NotFound<RawHtml<T>>;

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Serialized::defaults(Config::default()))
        .merge(Toml::file("config/server.toml").nested())
        .merge(Env::prefixed("MY_WEB_").global())
        .select(Profile::from_env_or("MY_WEB_PROFILE", "default"));

    let theme_dir = &figment
        .extract::<Config>()
        .expect("Failed to extract config")
        .theme_dir;

    let theme = &figment
        .extract::<Config>()
        .expect("Failed to extract config")
        .theme;

    let template = match load_all_templates(theme_dir, theme).await {
        Ok(templates) => Arc::new(RwLock::new(templates)),
        Err(e) => {
            eprintln!("Failed to load templates: {}", e);
            std::process::exit(1);
        }
    };

    let page_cache: PageCache = Arc::new(RwLock::new(HashMap::new()));

    if cfg!(debug_assertions) {
        println!("Running in debug mode");
    }

    let _rocket = rocket::custom(&figment)
        .attach(AdHoc::config::<Config>())
        .manage(template)
        .manage(page_cache)
        .mount("/", routes![index, page, blog, not_found])
        .mount(
            "/static",
            rocket::fs::FileServer::from(theme_dir.join(theme).join("static")),
        )
        .launch()
        .await?;
    Ok(())
}

async fn read_template(
    is_component: bool,
    template: &str,
    theme_dir: &Path,
    theme: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let template = match is_component {
        true => format!("components/{}", template),
        false => template.to_owned(),
    };
    read_to_string(
        theme_dir
            .join(theme)
            .join("templates")
            .join(template)
            .with_extension("hbs"),
    )
    .await
    .map_err(|e| e.into())
}

async fn load_all_templates(
    theme_dir: &Path,
    theme: &str,
) -> Result<
    HashMap<String, Result<String, Box<dyn Error + Send + Sync>>>,
    Box<dyn Error + Send + Sync>,
> {
    let templates = vec![
        ("main_layout", true),
        ("min_layout", true),
        ("navbar", true),
        ("overlay", true),
        ("blog_post", false),
        ("blog", false),
        ("projects", false),
        ("about", false),
        ("index", false),
        ("404", false),
    ];

    let mut template_pool = HashMap::new();

    for (name, is_component) in templates {
        let content = read_template(is_component, name, theme_dir, theme).await;
        template_pool.insert(name.to_string(), content);
    }

    Ok(template_pool)
}

#[derive(Deserialize, Serialize, Debug)]
struct Menu {
    name: String,
    url: String,
}
#[derive(Deserialize, Serialize, Debug)]
struct Menus(Vec<Menu>);

impl Default for Menus {
    fn default() -> Self {
        Menus(vec![
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
        ])
    }
}

impl From<Menus> for Json {
    fn from(menus: Menus) -> Self {
        json!(menus.0).into()
    }
}
impl TryFrom<Json> for Menus {
    type Error = serde_json::Error;
    fn try_from(json: Json) -> Result<Self, Self::Error> {
        serde_json::from_value(json.into()).map(Menus)
    }
}

fn make_data(menus: Json, title: &str) -> Map<String, Value> {
    let mut data = Map::new();
    data.insert("menus".to_string(), menus.into());
    data.insert("title".to_string(), to_json(title));
    data.insert("default_theme".to_string(), to_json("mocha"));
    data.insert("secondary_theme".to_string(), to_json("latte"));
    data.insert("parent".to_string(), to_json("layout"));
    data
}

async fn make_page(
    template: &State<TemplatePool>,
    page_name: &str,
    layout: &str,
    data: Map<String, Value>,
) -> Result<post::Html, Box<dyn Error + Send + Sync>> {
    let layout = layout.to_owned() + "_layout";
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("navbar", template.get_template("navbar").await?)?;
    handlebars.register_template_string("overlay", template.get_template("overlay").await?)?;
    handlebars.register_template_string("layout", template.get_template(&layout).await?)?;
    handlebars.register_template_string(page_name, template.get_template(page_name).await?)?;
    let hb = Html::new(handlebars.render(page_name, &data)?).minify()?;
    Ok(hb)
}

async fn get_or_generate_page(
    templates: &State<TemplatePool>,
    layout: &str,
    page_cache: &State<PageCache>,
    page_name: &str,
    data: Map<String, Value>,
    cache_duration: Duration,
    cache_id: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    if cfg!(debug_assertions) {
        let generated_page = make_page(templates, page_name, layout, data)
            .await?
            .minify()?
            .to_string();
        Ok(generated_page)
    } else {
        // Try to get the page from the cache
        {
            let cache = page_cache.read().await;
            if let Some((page, timestamp)) = cache.get(cache_id) {
                if timestamp.elapsed() < cache_duration.to_std()? {
                    return Ok(page.clone());
                }
            }
        }

        // If not in cache or expired, generate the page
        let generated_page = make_page(templates, page_name, layout, data)
            .await?
            .minify()?
            .to_string();

        // Store the generated page in the cache
        {
            let mut cache = page_cache.write().await;
            cache.insert(
                cache_id.to_string(),
                (generated_page.clone(), Instant::now()),
            );
        }
        Ok(generated_page)
    }
}

async fn make_404(template: &State<TemplatePool>, message: &str) -> RawHtml<String> {
    if message == "test fatal" {
        #[allow(unused_variables)]
        let template = TemplatePool::default();
    }
    let mut data = make_data(Menus::default().into(), "404");
    data.insert("message_404".to_string(), to_json(message));
    let html = make_page(template, "404", "min", data).await;
    match html {
        Ok(html) => RawHtml::from(html),
        Err(e) => RawHtml(format!(
            r#"<h1>404</h1>
            <h2>FATAL, multiple errors</h2>
            <p>{}</p>
            <p>{}</p>"#,
            message, e
        )),
    }
}

#[get("/404")]
async fn not_found(template: &State<TemplatePool>) -> RawHtml<String> {
    make_404(template, "test fatal").await
}

#[get("/")]
async fn index(
    template: &State<TemplatePool>,
    page_cache: &State<PageCache>,
) -> Result<RawHtml<String>, NotFound<String>> {
    let menus = Menus::default().into();
    let data = make_data(menus, "ISAALULA");
    let html = get_or_generate_page(
        template,
        "main",
        page_cache,
        "index",
        data,
        Duration::hours(1),
        "index",
    )
    .await;
    match html {
        Ok(html) => Ok(RawHtml(html.to_string())),
        Err(e) => Err(NotFound(e.to_string())),
    }
}

#[get("/<page>")]
async fn page(
    page: &str,
    template: &State<TemplatePool>,
    page_cache: &State<PageCache>,
) -> Result<Result<RawHtml<String>, Redirect>, NotFound<String>> {
    let menus = Menus::default().into();
    let data = make_data(menus, &page.to_uppercase());
    let page_title: [&str; 3] = ["blog", "projects", "about"];
    match page {
        "index" => Ok(Err(Redirect::to("/"))),
        p if page_title.contains(&p) => {
            let html = get_or_generate_page(
                template,
                "main",
                page_cache,
                page,
                data,
                Duration::hours(1),
                page,
            )
            .await;
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
    template: &State<TemplatePool>,
    page_cache: &State<PageCache>,
) -> Result<RawHtml<String>, Handle404<String>> {
    let menus = Menus::default().into();
    let blog_content = match read_to_string(format!("articles/blog/{}.md", blog_post)).await {
        Ok(s) => s,
        Err(e) => return Err(NotFound(Html::from(e).into())),
    };

    let html = match post::Markdown::new(blog_content).to_html(post::MarkdownType::Gfm) {
        Ok(h) => h.to_string(),
        Err(e) => return Err(NotFound(Html::from(e).into())),
    };
    let title = blog_post.snake_to_title_case();
    let mut data = make_data(menus, &title);
    data.insert("blog_post".to_string(), to_json(html));
    let html = get_or_generate_page(
        template,
        "main",
        page_cache,
        "blog_post",
        data,
        Duration::hours(1),
        &title,
    )
    .await;
    match html {
        Ok(html) => Ok(RawHtml(html.to_string())),
        Err(e) => Err(NotFound(Html::from(e).into())),
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

#[async_trait]
trait GetTemplate {
    async fn get_template(
        &self,
        template_name: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl GetTemplate for TemplatePool {
    async fn get_template(
        &self,
        template_name: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let templates = self.read().await;
        match templates.get(template_name) {
            Some(Ok(template)) => Ok(template.to_string()),
            Some(Err(e)) => Err(IoError::new(
                ErrorKind::Other,
                format!("Failed to read template '{}': {}", template_name, e),
            )
            .into()),
            None => Err(IoError::new(
                ErrorKind::NotFound,
                format!("Template '{}' not found", template_name),
            )
            .into()),
        }
    }
}
