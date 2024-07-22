mod menus;

use crate::config::Config;
use crate::post::PreviewArticle;
use crate::template::load_all_templates;
use crate::{
    post::{Html, Markdown, MarkdownType},
    render::{get_page, make_data, render, render_page, PageCache},
    template::TemplatePool,
    theme, Error, Result, SnakeToTitleCase, TitleCase,
};
use chrono::Duration;
use figment::Figment;
use handlebars::to_json;
use rocket::fairing::AdHoc;
use rocket::fs::NamedFile;
use rocket::{
    get,
    response::{content::RawHtml, status::NotFound},
    routes, Build, Rocket, State,
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::read_to_string;
pub use menus::{Menu, Menus};
type StdResult<T, E> = std::result::Result<T, E>;
pub async fn launch(figment: &Figment) -> Result<Rocket<Build>> {
    let theme_dir = &figment
        .extract::<Config>()
        .expect("Failed to extract config")
        .theme_dir;

    let theme = &figment
        .extract::<Config>()
        .expect("Failed to extract config")
        .theme;

    let template = load_all_templates(theme_dir, theme)
        .await
        .map(|templates| TemplatePool::from(false, templates))?;

    let menus = Menus::default();

    let page_cache: PageCache = PageCache::new(false);

    if cfg!(debug_assertions) {
        let markdown = Markdown::from(read_to_string("articles/blog/ant_dilemma.md").await?);
        let article_prev = &markdown.preview().await?;
        let theme = theme::Theme::read(
            &theme_dir
                .join(theme.as_ref())
                .join("meta")
                .with_extension("toml"),
        )
        .await?;
        println!("Preview: {:#?}", article_prev);
        // println!("Templates: {:#?}", template.read().await);
        println!("Theme:\n{:#?}", theme);
    }

    let rocket = rocket::custom(figment)
        .attach(AdHoc::config::<Config>())
        .manage(template)
        .manage(page_cache)
        .manage(menus)
        .mount("/", routes![index, static_files, blog, pages, not_found]);

    Ok(rocket)
}

pub async fn make_404(
    template_pool: &State<TemplatePool>,
    message: &str,
    menu: &State<Menus>,
) -> Html {
    if message == "test" {
        #[allow(unused_variables)]
        let template = TemplatePool::new(false);
    }
    let template_list = Box::new(vec![
        ("default", "default"),
        ("navbar", "navbar"),
        ("overlay", "overlay"),
        ("layout", "layout"),
        ("article", "404"),
    ]);
    let data_list = [
        ("parent".to_string(), to_json("layout")),
        ("site_name".to_string(), to_json("ISAALULA")),
        ("page_title".to_string(), to_json("Not Found")),
        ("layout_min".to_string(), to_json(true)),
        ("menus".to_string(), to_json(menu.get())),
        ("default_theme".to_string(), to_json("mocha")),
        ("secondary_theme".to_string(), to_json("latte")),
        ("message".to_string(), to_json(message)),
    ];
    let data = make_data(&data_list);
    let html = render("default", template_pool, &template_list, data).await;
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
pub async fn not_found(template: &State<TemplatePool>, menus: &State<Menus>) -> RawHtml<String> {
    RawHtml(make_404(template, "test fatal", menus).await.to_string())
}

#[get("/")]
pub async fn index(
    template_pool: &State<TemplatePool>,
    page_cache: &State<PageCache>,
    menus: &State<Menus>,
) -> StdResult<RawHtml<Arc<str>>, NotFound<RawHtml<String>>> {
    let page = "index";

    if !cfg!(debug_assertions) {
        let from_cache = get_page(page_cache, Duration::hours(1), page)
            .await
            .map_err(|e| e.to_string());
        let cached: Option<Arc<str>> = match from_cache {
            Ok(o) => o,
            Err(e) => {
                return Err(NotFound(RawHtml(
                    make_404(template_pool, &e.to_string(), menus)
                        .await
                        .to_string(),
                )))
            }
        };

        if let Some(cached) = cached {
            return Ok(RawHtml(cached));
        }
    }

    let template_list = Box::new(vec![
        ("default", "default"),
        ("navbar", "navbar"),
        ("overlay", "overlay"),
        ("layout", "layout"),
        ("article", "blog"),
    ]);
    let data_list = [
        ("parent".to_string(), to_json("layout")),
        ("site_name".to_string(), to_json("ISAALULA")),
        ("page_title".to_string(), to_json("Isa Al-Ula")),
        ("layout_min".to_string(), to_json(false)),
        ("menus".to_string(), to_json(menus.get())),
        ("default_theme".to_string(), to_json("mocha")),
        ("secondary_theme".to_string(), to_json("latte")),
        ("article".to_string(), to_json(r#"<h1>INDEX</h1>"#)),
    ];
    let data = make_data(&data_list);
    let html = render_page(
        "default",
        template_pool,
        &template_list,
        data,
        page_cache,
        page,
    )
    .await;
    match html {
        Ok(html) => Ok(RawHtml(html)),
        Err(e) => Err(NotFound(RawHtml(
            make_404(template_pool, &e.to_string(), menus)
                .await
                .to_string(),
        ))),
    }
}

#[get("/<page>")]
pub async fn pages(
    page: &str,
    template_pool: &State<TemplatePool>,
    page_cache: &State<PageCache>,
    menus: &State<Menus>,
) -> StdResult<RawHtml<Arc<str>>, NotFound<RawHtml<String>>> {
    let page_title: [&str; 3] = ["blog", "projects", "about"];
    match page {
        p if page_title.contains(&p) => {
            if !cfg!(debug_assertions) {
                let from_cache = get_page(page_cache, Duration::hours(1), page)
                    .await
                    .map_err(|e| e.to_string());
                let cached: Option<Arc<str>> = match from_cache {
                    Ok(o) => o,
                    Err(e) => {
                        return Err(NotFound(RawHtml(
                            make_404(template_pool, &e.to_string(), menus)
                                .await
                                .to_string(),
                        )))
                    }
                };

                if let Some(cached) = cached {
                    return Ok(RawHtml(cached));
                }
            }
            let template_list = Box::new(vec![
                ("default", "default"),
                ("navbar", "navbar"),
                ("overlay", "overlay"),
                ("layout", "layout"),
                ("article", "blog"),
            ]);
            let mut data_list = vec![
                ("parent".to_string(), to_json("layout")),
                ("site_name".to_string(), to_json("ISAALULA")),
                ("layout_min".to_string(), to_json(false)),
                ("menus".to_string(), to_json(menus.get())),
                ("default_theme".to_string(), to_json("mocha")),
                ("secondary_theme".to_string(), to_json("latte")),
            ];
            data_list.push((
                "article".to_string(),
                to_json(format!("<h1>{}</h1>", p.title_case())),
            ));
            data_list.push(("page_title".to_string(), to_json(p.title_case())));
            let data = make_data(&data_list);
            let html = render_page(
                "default",
                template_pool,
                &template_list,
                data,
                page_cache,
                page,
            )
            .await;
            match html {
                Ok(html) => Ok(RawHtml(html)),
                Err(e) => Err(NotFound(RawHtml(
                    make_404(template_pool, &e.to_string(), menus)
                        .await
                        .to_string(),
                ))),
            }
        }
        _ => Err(NotFound(RawHtml(
            make_404(template_pool, "page not found", menus)
                .await
                .to_string(),
        ))),
    }
}

#[get("/<page>/<article>")]
pub async fn blog(
    page: &str,
    article: &str,
    template_pool: &State<TemplatePool>,
    page_cache: &State<PageCache>,
    menus: &State<Menus>,
) -> StdResult<RawHtml<Arc<str>>, NotFound<RawHtml<String>>> {
    match page {
        "blog" => {
            let blog_content = match read_to_string(format!("articles/blog/{}.md", article)).await {
                Ok(s) => s,
                Err(e) => {
                    return Err(NotFound(RawHtml(
                        make_404(template_pool, &e.to_string(), menus)
                            .await
                            .to_string(),
                    )))
                }
            };
            let title = article.snake_to_title_case();

            if !cfg!(debug_assertions) {
                let from_cache = get_page(page_cache, Duration::hours(1), &title)
                    .await
                    .map_err(|e| e.to_string());
                let cached: Option<Arc<str>> = match from_cache {
                    Ok(o) => o,
                    Err(e) => {
                        return Err(NotFound(RawHtml(
                            make_404(template_pool, &e.to_string(), menus)
                                .await
                                .to_string(),
                        )))
                    }
                };

                if let Some(cached) = cached {
                    return Ok(RawHtml(cached));
                }
            }

            let html = match Markdown::new(blog_content).to_html(MarkdownType::Gfm) {
                Ok(h) => h.to_string(),
                Err(e) => {
                    return Err(NotFound(RawHtml(
                        make_404(template_pool, &e.to_string(), menus)
                            .await
                            .to_string(),
                    )))
                }
            };
            let template_list = Box::new(vec![
                ("default", "default"),
                ("navbar", "navbar"),
                ("overlay", "overlay"),
                ("layout", "layout"),
                ("article", "blog"),
            ]);
            let data_list = [
                ("parent".to_string(), to_json("layout")),
                ("site_name".to_string(), to_json("ISAALULA")),
                ("page_title".to_string(), to_json(&title)),
                ("layout_min".to_string(), to_json(false)),
                ("menus".to_string(), to_json(menus.get())),
                ("default_theme".to_string(), to_json("mocha")),
                ("secondary_theme".to_string(), to_json("latte")),
                ("article".to_string(), to_json(html)),
            ];
            let data = make_data(&data_list);
            let html = render_page(
                "default",
                template_pool,
                &template_list,
                data,
                page_cache,
                &title,
            )
            .await;
            match html {
                Ok(html) => Ok(RawHtml(html)),
                Err(e) => Err(NotFound(
                    make_404(template_pool, &e.to_string(), menus).await.into(),
                )),
            }
        }
        _ => Err(NotFound(RawHtml(String::from("404")))),
    }
}

#[get("/static/<file..>")]
pub async fn static_files(
    file: PathBuf,
    config: &State<Config>,
) -> StdResult<NamedFile, NotFound<RawHtml<String>>> {
    let theme_dir = &config.theme_dir;
    let theme = &config.theme;
    let file = theme_dir.join(theme.as_ref()).join("static").join(file);
    NamedFile::open(file).await.map_err(|e|NotFound(RawHtml(e.to_string())))
}
