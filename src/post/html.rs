use std::{fmt, io};
use std::error::Error;
use std::fmt::Display;
use ammonia::{clean, is_html};
use minify_html::minify;
use rocket::response::content::RawHtml;
use super::Join;
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