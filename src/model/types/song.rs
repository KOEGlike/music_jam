use crate::model::types::*;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    /// is some if it is in the jam, is none if for example it came from search
    pub id: Option<String>,
    pub spotify_id: String,
    ///Some if requested by the user who added the song, none if requested by the host or a user who didn't add the song
    pub user_id: Option<String>,
    pub name: String,
    pub artists: Vec<String>,
    pub album: String,
    pub duration: u32,
    pub image_url: String,
    pub votes: Vote,
}

pub trait ToVotes {
    fn to_votes(self) -> Option<Votes>;
}

impl ToVotes for Vec<Song> {
    fn to_votes(self) -> Option<Votes> {
        if self.iter().map(|s| s.id.is_none()).contains(&true) {
            return None;
        } else {
            Some(self.into_iter().map(|s| (s.id.unwrap(), s.votes)).collect())
        }
    }
}
