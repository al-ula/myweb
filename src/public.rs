use chrono::Duration;
use handlebars::to_json;
use rocket::{
    get,
    response::{content::RawHtml, status::NotFound, Redirect},
    State,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs::read_to_string;

use crate::{
    json::Json,
    page::{get_or_generate_page, make_data, make_page, PageCache},
    post::{Html, Markdown, MarkdownType},
    template::TemplatePool,
    SnakeToTitleCase, TitleCase,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Menu {
    pub name: String,
    pub url: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Menus(Vec<Menu>);

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

pub async fn make_404(template_pool: &State<TemplatePool>, message: &str) -> Html {
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
    let html = make_page("default", template_pool, &template_list, data).await;
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
pub async fn not_found(template: &State<TemplatePool>) -> RawHtml<String> {
    RawHtml(make_404(template, "test fatal").await.to_string())
}

#[get("/")]
pub async fn index(
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
pub async fn pages(
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
pub async fn blog(
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

    let html = match Markdown::new(blog_content).to_html(MarkdownType::Gfm) {
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
