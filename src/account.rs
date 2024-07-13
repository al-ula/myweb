use ulid::Ulid;

use crate::json::Json;

pub struct Account {
    pub id: Ulid,
    pub username: String,
    pub email: String,
    pub secret: String,
    pub privilege: Privilege,
    pub google_auth: Option<Json>,
}

pub enum Privilege {
    Admin,
    Moderator,
    Writer,
    Reader,
}

pub struct Privileges {
    pub server_config: bool,
    pub site_config: bool,
    pub content_authority: ContentAuthority,
    pub comment: bool,
}

pub enum ContentAuthority {
    All,
    Oneself,
    None,
    Except(Vec<Ulid>),
    Specific(Vec<Ulid>),
}

pub trait GetPrivileges {
    fn get_privileges(&self) -> Privileges;
}

impl GetPrivileges for Account {
    fn get_privileges(&self) -> Privileges {
        self.privilege.get_privileges()
    }
}

impl GetPrivileges for Privilege {
    fn get_privileges(&self) -> Privileges {
        match self {
            Privilege::Admin => Privileges {
                server_config: true,
                site_config: true,
                content_authority: ContentAuthority::All,
                comment: true,
            },
            Privilege::Moderator => Privileges {
                server_config: false,
                site_config: true,
                content_authority: ContentAuthority::All,
                comment: true,
            },
            Privilege::Writer => Privileges {
                server_config: false,
                site_config: false,
                content_authority: ContentAuthority::Oneself,
                comment: true,
            },
            Privilege::Reader => Privileges {
                server_config: false,
                site_config: false,
                content_authority: ContentAuthority::None,
                comment: true,
            },
        }
    }
}
