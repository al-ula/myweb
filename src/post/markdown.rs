use super::{Html, Join};
use crate::StringCutter;
use async_trait::async_trait;
use markdown::{mdast, to_html_with_options};
use crate::Error;
use std::fmt::Display;

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
impl From<Markdown> for String {
    fn from(markdown: Markdown) -> Self {
        markdown.0
    }
}

impl Markdown {
    pub fn new(content: String) -> Markdown {
        Markdown(content)
    }
    pub fn to_html(&self, type_: MarkdownType) -> Result<Html, Error> {
        match type_ {
            MarkdownType::Common => {
                match to_html_with_options(&self.0, &markdown::Options::default()) {
                    Ok(html) => Ok(Html::from(html)),
                    Err(e) => Err(e.to_string().into()),
                }
            }
            MarkdownType::Gfm => match to_html_with_options(&self.0, &markdown::Options::gfm()) {
                Ok(html) => Ok(Html::from(html)),
                Err(e) => Err(e.to_string().into()),
            },
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ArticlePrev {
    title: String,
    body: String,
}

#[async_trait]
pub trait PreviewArticle {
    async fn preview(&self) -> Result<ArticlePrev, Error>;
}

#[async_trait]
impl PreviewArticle for Markdown {
    async fn preview(&self) -> Result<ArticlePrev, Error> {
        let ast = match markdown::to_mdast(&self.to_string(), &Default::default()) {
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
                    None => return Err("Failed to find heading".to_string().into()),
                },
                None => return Err("Failed to parse article".to_string().into()),
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
                    Some(None) | None => return Err("Failed to find paragraph".to_string().into()),
                },
                None => return Err("Failed to parse article".to_string().into()),
            },
        };
        Ok(art)
    }
}

#[allow(dead_code)]
pub enum MarkdownType {
    Common,
    Gfm,
}
