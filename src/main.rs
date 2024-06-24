mod post;
use post::{article::Article, Markdown, MarkdownType};

#[tokio::main]
async fn main() {
    // println!("{:?}", Article::default());
    // let markdown: Markdown = r##" # Test "##.to_string().into();
    // let html = markdown.to_html(MarkdownType::Common).unwrap();
}
