pub mod article;
mod html;
mod markdown;

pub use html::Html;
pub use markdown::*;

#[allow(dead_code)]
pub trait Join<T> {
    fn join(&self, other: &T) -> Self;
}
