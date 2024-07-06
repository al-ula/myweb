pub mod article;

use ammonia::{clean, is_html};
use chrono::{DateTime, Utc};
use markdown::to_html_with_options;
use minify_html::minify;
use rocket::response::content::RawHtml;
use serde_json::Value;
use std::{
    error::Error,
    fmt::{self, Display},
    io,
};
use ulid::Ulid;

#[derive(Default, Clone, Debug)]
pub struct Html(String);

#[allow(dead_code)]
trait Join<T> {
    fn join(&self, other: &T) -> Self;
}
impl Display for Html {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<String> for Html {
    fn from(content: String) -> Self {
        Html(content)
    }
}
impl From<fmt::Error> for Html {
    fn from(e: fmt::Error) -> Self {
        Html(format!("<h1>{:?}<h1>", e))
    }
}
impl From<io::Error> for Html {
    fn from(e: io::Error) -> Self {
        Html(format!("<h1>{:?}<h1>", e))
    }
}
impl From<Box<dyn Error + Send + Sync>> for Html {
    fn from(e: Box<dyn Error + Send + Sync>) -> Self {
        Html(format!("<h1>{:?}<h1>", e))
    }
}
impl From<Html> for RawHtml<String> {
    fn from(html: Html) -> Self {
        RawHtml(html.0)
    }
}

impl Join<Html> for Html {
    fn join(&self, other: &Html) -> Self {
        Html(format!("{}{}", self.0, other.0))
    }
}

impl Join<String> for Html {
    fn join(&self, other: &String) -> Self {
        Html(format!("{}{}", self.0, other))
    }
}

impl Join<&str> for Html {
    fn join(&self, other: &&str) -> Self {
        Html(format!("{}{}", self.0, other))
    }
}
impl Join<RawHtml<String>> for Html {
    fn join(&self, other: &RawHtml<String>) -> Self {
        Html(format!("{}{}", self.0, other.0))
    }
}
impl Html {
    pub fn new(content: String) -> Html {
        Html(content)
    }
    pub fn minify(&self) -> Result<Html, Box<dyn Error + Send + Sync>> {
        let cfg = minify_html::Cfg {
            minify_js: true,
            ..Default::default()
        };
        match String::from_utf8(minify(self.to_string().as_bytes(), &cfg)) {
            Ok(html) => Ok(Html::from(html)),
            Err(e) => Err(Box::new(e)),
        }
    }
    pub fn validate(&self) -> bool {
        is_html(&self.0)
    }
    pub fn sanitize(&self) -> Html {
        Html(clean(&self.0))
    }
    // pub fn encaps(&self) -> Html {}
}

#[derive(Default, Clone, Debug)]
pub struct Markdown(String);
impl Display for Markdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<String> for Markdown {
    fn from(content: String) -> Self {
        Markdown(content)
    }
}
impl Markdown {
    pub fn new(content: String) -> Markdown {
        Markdown(content)
    }
    pub fn to_html(&self, type_: MarkdownType) -> Result<Html, Box<dyn Error + Send + Sync>> {
        match type_ {
            MarkdownType::Common => {
                match markdown::to_html_with_options(&self.0, &markdown::Options::default()) {
                    Ok(html) => Ok(Html(html)),
                    Err(e) => Err(e.to_string().into()),
                }
            }
            MarkdownType::Gfm => match to_html_with_options(&self.0, &markdown::Options::gfm()) {
                Ok(html) => Ok(Html(html)),
                Err(e) => Err(e.to_string().into()),
            },
        }
    }
}

#[allow(dead_code)]
pub enum MarkdownType {
    Common,
    Gfm,
}

#[derive(Default, Clone, Debug)]
pub struct Json(Value);
impl Display for Json {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<Value> for Json {
    fn from(content: Value) -> Self {
        Json(content)
    }
}
impl Json {
    pub fn new(content: Value) -> Json {
        Json(content)
    }
    pub fn from_str(content: &str) -> Result<Json, Box<dyn Error + Send + Sync>> {
        let value = serde_json::from_str(content);
        match value {
            Ok(value) => Ok(Json(value)),
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl From<Json> for Value {
    fn from(val: Json) -> Self {
        val.0
    }
}
