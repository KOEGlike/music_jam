pub type JamId = String;

#[derive(Debug, Clone)]
pub struct Id {
    pub id: String,
    pub jam_id: String,
}

#[derive(Clone, Debug)]
pub enum IdType {
    Host(Id),
    User(Id),
}

#[allow(dead_code)]
impl IdType {
    pub fn is_host(&self) -> bool {
        matches!(self, IdType::Host { .. })
    }
    pub fn is_user(&self) -> bool {
        matches!(self, IdType::User { .. })
    }
    pub fn id(&self) -> &str {
        match self {
            IdType::Host(id) => &id.id,
            IdType::User(id) => &id.id,
        }
    }
    pub fn jam_id(&self) -> &str {
        match self {
            IdType::Host(id) => &id.jam_id,
            IdType::User(id) => &id.jam_id,
        }
    }
}