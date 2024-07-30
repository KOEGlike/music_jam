use serde::{Serialize, Deserialize};
use crate::general::types::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub id: String,
    ///Some if requested by the user who added the song, none if requested by the host or a user who didn't add the song
    pub user_id: Option<String>,
    pub name: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration: u16,
    pub image_url: String,
    pub votes: Vote,
}

pub trait ToVotes {
    fn to_votes(&self) -> Votes;
}

impl ToVotes for Vec<Song> {
    fn to_votes(&self) -> Votes {
        self.iter()
            .map(|song| (song.id.clone(), song.votes))
            .collect()
    }
}