use serde::{Serialize, Deserialize};
use super::super::Song;
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub songs: Vec<Song>,
    pub search_id: String,
}
