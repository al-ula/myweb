use std::sync::Arc;
use rocket::serde::{Deserialize, Serialize};
use serde_json::json;
use crate::json::Json;

#[derive(Deserialize, Serialize, Debug)]
pub struct Menu {
    pub name: String,
    pub url: String,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct Menus(Arc<[Menu]>);

impl Default for Menus {
    fn default() -> Self {
        Menus(
            vec![
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
            ]
                .into(),
        )
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

impl Menus {
    pub fn get(&self) -> &Menu {
        &self.0[0]
    }
}