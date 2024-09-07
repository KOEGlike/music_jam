pub type JamId = String;

#[derive(Debug, Clone)]
pub struct Id {
    pub id: IdType,
    pub jam_id: String,
}

impl Id {
    pub fn new(id: IdType, jam_id: String) -> Self {
        Self { id, jam_id }
    }

    pub fn jam_id(&self) -> &str {
        &self.jam_id
    }

    pub fn id(&self) -> Option<&str> {
        self.id.id()
    }

    pub fn is_host(&self) -> bool {
        self.id.is_host()
    }

    pub fn is_user(&self) -> bool {
        self.id.is_user()
    }

    pub fn is_general(&self) -> bool {
        self.id.is_general()
    }

    pub fn id_type(&self) -> &IdType {
        &self.id
    }
}

#[derive(Clone, Debug)]
pub enum IdType {
    Host(String),
    User(String),
    General
}

impl IdType {
    pub fn id(&self) -> Option<&str> {
        match self {
            IdType::Host(id) => Some(id),
            IdType::User(id) => Some(id),
            IdType::General => None
        }
    }

    pub fn is_host(&self) -> bool {
        matches!(self, IdType::Host(_))
    }

    pub fn is_user(&self) -> bool {
        matches!(self, IdType::User(_))
    }

    pub fn is_general(&self) -> bool {
        matches!(self, IdType::General)
    }
}