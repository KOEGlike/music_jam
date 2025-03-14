use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    KickUser { user_id: String },
    AddSong { song_id: String },
    RemoveSong { song_id: String },
    AddVote { song_id: String },
    RemoveVote { song_id: String },
    Search { query: String, id: String },
    Position { percentage: f32 },
}
