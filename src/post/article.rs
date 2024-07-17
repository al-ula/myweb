use std::fmt::Display;
use chrono::{DateTime, Utc};
use ulid::Ulid;
use super::{Html, Markdown};


#[derive(Default, Clone, Debug)]
pub struct Article {
    id: Ulid,
    pub title: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub content: Content,
}

#[derive(Clone, Debug)]
pub enum Content {
    Markdown(Markdown),
    Html(Html),
}

impl Display for Article {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Article {
    pub fn new(
        id: Ulid,
        title: String,
        author: String,
        timestamp: DateTime<Utc>,
        content: Content,
    ) -> Article {
        Article {
            id,
            title,
            author,
            timestamp,
            content,
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
    }
}

impl Default for Content {
    fn default() -> Self {
        Content::Html(Html::from(String::new()))
    }
}
