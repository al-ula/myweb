mod post;
mod theme;
use async_trait::async_trait;
use chrono::{format, Duration};
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment, Profile,
};
use handlebars::{to_json, Handlebars};
use markdown::mdast;
use post::{Html, Join, Json, Markdown};
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
        let markdown = Markdown::from(
            read_to_string("articles/blog/ant_dilemma.md")
                .await
                .unwrap(),
        );
        let article_prev = preview_article(&markdown).await.unwrap();
        println!("Preview: {:#?}", article_prev);
        println!("Templates: {:#?}", template.read().await);
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
        ("layout", true),
        ("navbar", true),
        ("overlay", true),
        ("blog", true),
        ("404", true),
        ("default", false),
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

#[derive(Debug)]
struct ArticlePrev {
    title: String,
    body: String,
}

async fn preview_article(article: &Markdown) -> Result<ArticlePrev, Box<dyn Error + Send + Sync>> {
    let ast = match markdown::to_mdast(&article.to_string(), &Default::default()) {
        Ok(a) => a,
        Err(e) => return Err(e.to_string().into()),
    };

    let art = ArticlePrev {
        title: match ast.children() {
            Some(r) => match r.iter().find_map(|r| match r {
                mdast::Node::Heading(h) => match h.depth {
                    1 => h.children.iter().find_map(|n| match n {
                        mdast::Node::Text(t) => Some(t.value.to_string()),
                        _ => None,
                    }),
                    _ => None,
                },
                _ => None,
            }) {
                Some(s) => s,
                None => return Err("Failed to find heading".into()),
            },
            None => return Err("Failed to parse article".into()),
        },
        body: match ast.children() {
            Some(r) => match r.iter().find_map(|r| match r {
                mdast::Node::Paragraph(p) => Some(p.children.iter().find_map(|n| match n {
                    mdast::Node::Text(t) => Some(t.value.to_string()),
                    _ => None,
                })),
                _ => None,
            }) {
                Some(Some(s)) => s.cut_to_length(200).join(&"...".to_string()),
                Some(None) | None => return Err("Failed to find paragraph".into()),
            },
            None => return Err("Failed to parse article".into()),
        },
    };
    Ok(art)
}

fn make_data(data_list: &[(String, Value)]) -> Map<String, Value> {
    let data_list = data_list.to_owned();
    let mut data = Map::new();
    for datum in data_list {
        data.insert(datum.0, datum.1);
    }
    data
}

async fn make_page(
    page_template: &str,
    template_pool: &State<TemplatePool>,
    template_list: &[(&str, &str)],
    data: Map<String, Value>,
) -> Result<post::Html, Box<dyn Error + Send + Sync>> {
    let mut handlebars = Handlebars::new();
    for t in template_list.iter() {
        handlebars.register_template_string(t.0, template_pool.get_template(t.1).await?)?;
    }
    let hb = Html::new(handlebars.render(page_template, &data)?).minify()?;
    Ok(hb)
}

async fn get_or_generate_page(
    page_template: &str,
    template_pool: &State<TemplatePool>,
    template_list: &[(&str, &str)],
    data: Map<String, Value>,
    page_cache: &State<PageCache>,
    cache_duration: Duration,
    cache_id: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    if cfg!(debug_assertions) {
        let generated_page = make_page(page_template, template_pool, template_list, data)
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
        let generated_page = make_page(page_template, template_pool, template_list, data)
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

async fn make_404(template_pool: &State<TemplatePool>, message: &str) -> Html {
    if message == "test" {
        #[allow(unused_variables)]
        let template = TemplatePool::default();
    }
    let template_list = vec![
        ("default", "default"),
        ("navbar", "navbar"),
        ("overlay", "overlay"),
        ("layout", "layout"),
        ("article", "404"),
    ];
    let data_list = [
        ("parent".to_string(), to_json("layout")),
        ("site_name".to_string(), to_json("ISAALULA")),
        ("page_title".to_string(), to_json("Not Found")),
        ("layout_min".to_string(), to_json(true)),
        ("menus".to_string(), to_json(Menus::default())),
        ("default_theme".to_string(), to_json("mocha")),
        ("secondary_theme".to_string(), to_json("latte")),
        ("message".to_string(), to_json(message)),
    ];
    let data = make_data(&data_list);
    let html = make_page("404", template_pool, &template_list, data).await;
    match html {
        Ok(html) => html,
        Err(e) => Html::from(format!(
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
    RawHtml(make_404(template, "test fatal").await.to_string())
}

#[get("/")]
async fn index(
    template_pool: &State<TemplatePool>,
    page_cache: &State<PageCache>,
) -> Result<RawHtml<String>, NotFound<String>> {
    let template_list = vec![
        ("default", "default"),
        ("navbar", "navbar"),
        ("overlay", "overlay"),
        ("layout", "layout"),
        ("article", "blog"),
    ];
    let data_list = [
        ("parent".to_string(), to_json("layout")),
        ("site_name".to_string(), to_json("ISAALULA")),
        ("page_title".to_string(), to_json("Isa Al-Ula")),
        ("layout_min".to_string(), to_json(false)),
        ("menus".to_string(), to_json(Menus::default())),
        ("default_theme".to_string(), to_json("mocha")),
        ("secondary_theme".to_string(), to_json("latte")),
        ("article".to_string(), to_json(r#"<h1>INDEX</h1>"#)),
    ];
    let data = make_data(&data_list);
    let html = get_or_generate_page(
        "default",
        template_pool,
        &template_list,
        data,
        page_cache,
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
    template_pool: &State<TemplatePool>,
    page_cache: &State<PageCache>,
) -> Result<Result<RawHtml<String>, Redirect>, NotFound<RawHtml<String>>> {
    let template_list = vec![
        ("default", "default"),
        ("navbar", "navbar"),
        ("overlay", "overlay"),
        ("layout", "layout"),
        ("article", "blog"),
    ];
    let mut data_list = vec![
        ("parent".to_string(), to_json("layout")),
        ("site_name".to_string(), to_json("ISAALULA")),
        ("layout_min".to_string(), to_json(false)),
        ("menus".to_string(), to_json(Menus::default())),
        ("default_theme".to_string(), to_json("mocha")),
        ("secondary_theme".to_string(), to_json("latte")),
    ];

    let page_title: [&str; 3] = ["blog", "projects", "about"];
    match page {
        "index" => Ok(Err(Redirect::to("/"))),
        p if page_title.contains(&p) => {
            data_list.push((
                "article".to_string(),
                to_json(format!("<h1>{}</h1>", p.title_case())),
            ));
            data_list.push(("page_title".to_string(), to_json(p.title_case())));
            let data = make_data(&data_list);
            let html = get_or_generate_page(
                "default",
                template_pool,
                &template_list,
                data,
                page_cache,
                Duration::hours(1),
                page,
            )
            .await;
            match html {
                Ok(html) => Ok(Ok(RawHtml(html.to_string()))),
                Err(e) => Err(NotFound(RawHtml(
                    make_404(template_pool, &e.to_string()).await.to_string(),
                ))),
            }
        }
        _ => Err(NotFound(RawHtml(
            make_404(template_pool, "page not found").await.to_string(),
        ))),
    }
}

#[get("/blog/<blog_post>")]
async fn blog(
    blog_post: &str,
    template_pool: &State<TemplatePool>,
    page_cache: &State<PageCache>,
) -> Result<RawHtml<String>, NotFound<RawHtml<String>>> {
    let blog_content = match read_to_string(format!("articles/blog/{}.md", blog_post)).await {
        Ok(s) => s,
        Err(e) => {
            return Err(NotFound(RawHtml(
                make_404(template_pool, &e.to_string()).await.to_string(),
            )))
        }
    };

    let html = match post::Markdown::new(blog_content).to_html(post::MarkdownType::Gfm) {
        Ok(h) => h.to_string(),
        Err(e) => {
            return Err(NotFound(RawHtml(
                make_404(template_pool, &e.to_string()).await.to_string(),
            )))
        }
    };
    let title = blog_post.snake_to_title_case();
    let template_list = vec![
        ("default", "default"),
        ("navbar", "navbar"),
        ("overlay", "overlay"),
        ("layout", "layout"),
        ("article", "blog"),
    ];
    let data_list = [
        ("parent".to_string(), to_json("layout")),
        ("site_name".to_string(), to_json("ISAALULA")),
        ("page_title".to_string(), to_json(&title)),
        ("layout_min".to_string(), to_json(false)),
        ("menus".to_string(), to_json(Menus::default())),
        ("default_theme".to_string(), to_json("mocha")),
        ("secondary_theme".to_string(), to_json("latte")),
        ("article".to_string(), to_json(html)),
    ];
    let data = make_data(&data_list);
    let html = get_or_generate_page(
        "default",
        template_pool,
        &template_list,
        data,
        page_cache,
        Duration::hours(1),
        &title,
    )
    .await;
    match html {
        Ok(html) => Ok(RawHtml(html.to_string())),
        Err(e) => Err(NotFound(RawHtml(
            make_404(template_pool, &e.to_string()).await.to_string(),
        ))),
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
trait TitleCase {
    fn title_case(&self) -> String;
}

impl TitleCase for &str {
    fn title_case(&self) -> String {
        self.split(' ')
            .map(|s| {
                s.chars().next().unwrap().to_uppercase().collect::<String>() + s.get(1..).unwrap()
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl TitleCase for String {
    fn title_case(&self) -> String {
        self.as_str().title_case()
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

pub trait StringCutter {
    fn cut_to_length(&self, max_length: usize) -> String;
}

impl StringCutter for String {
    fn cut_to_length(&self, max_length: usize) -> String {
        if self.len() <= max_length {
            self.clone()
        } else {
            self.chars().take(max_length).collect::<String>()
        }
    }
}

impl StringCutter for str {
    fn cut_to_length(&self, max_length: usize) -> String {
        if self.len() <= max_length {
            self.to_string()
        } else {
            self.chars().take(max_length).collect::<String>()
        }
    }
}

impl Join<String> for String {
    fn join(&self, other: &std::string::String) -> String {
        format!("{}{}", self, &other)
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
