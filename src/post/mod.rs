pub mod article;

use ammonia::{clean, is_html};
use chrono::{DateTime, Utc};
use markdown::to_html_with_options;
use minify_html::minify;
use serde_json::Value;
use std::fmt::{Display, Error};
use ulid::Ulid;

#[derive(Default, Clone, Debug)]
pub struct Html(String);

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
impl Html {
    pub fn new(content: String) -> Html {
        Html(content)
    }
    pub fn minify(&self) -> Result<Html, Error> {
        let cfg = minify_html::Cfg {
            minify_js: true,
            ..Default::default()
        };
        let html = minify(self.to_string().as_bytes(), &cfg);
        Ok(Html(String::from_utf8(html).unwrap()))
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
    pub fn to_html(&self, type_: MarkdownType) -> Result<Html, Error> {
        match type_ {
            MarkdownType::Common => {
                match markdown::to_html_with_options(&self.0, &markdown::Options::default()) {
                    Ok(html) => Ok(Html(html)),
                    Err(_) => Err(std::fmt::Error),
                }
            }
            MarkdownType::Gfm => match to_html_with_options(&self.0, &markdown::Options::gfm()) {
                Ok(html) => Ok(Html(html)),
                Err(_) => Err(std::fmt::Error),
            },
        }
    }
}

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
    pub fn from_str(content: &str) -> Result<Json, Error> {
        let value = serde_json::from_str(content);
        match value {
            Ok(value) => Ok(Json(value)),
            Err(_) => Err(std::fmt::Error),
        }
    }
}
