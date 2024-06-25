mod post;
use handlebars::{to_json, Handlebars};
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Serialize, Debug)]
struct Menu {
    name: String,
    url: String,
}

fn make_data(menus: Vec<Menu>) -> Map<String, Value> {
    let mut data = Map::new();
    data.insert("menus".to_string(), to_json(menus));
    data.insert("title".to_string(), to_json("ISAALULA"));
    data.insert("default_theme".to_string(), to_json("mocha"));
    data.insert("secondary_theme".to_string(), to_json("latte"));
    data.insert("overlay".to_string(), to_json("overlay"));
    data
}

async fn index() -> Result<post::Html, Box<dyn std::error::Error>> {
    let menus: Vec<Menu> = vec![
        Menu {
            name: "Home".to_string(),
            url: "/".to_string(),
        },
        Menu {
            name: "Blog".to_string(),
            url: "/blog".to_string(),
        },
        Menu {
            name: "About".to_string(),
            url: "/about".to_string(),
        },
    ];

    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("navbar", "templates/components/navbar.hbs")?;
    handlebars.register_template_file("overlay", "templates/components/overlay.hbs")?;
    handlebars.register_template_file("layout", "templates/components/main_layout.hbs")?;
    let data = make_data(menus);
    let hb = post::Html::new(handlebars.render("layout", &data)?);
    Ok(hb)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hb = index().await?;
    println!("{}", hb.minify()?);
    Ok(())
}
