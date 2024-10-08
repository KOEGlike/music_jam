use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Jam {
    pub id: String,
    pub name: String,
    pub max_song_count: u8,
}

