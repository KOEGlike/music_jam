use super::super::Song;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub songs: Vec<Song>,
    pub search_id: String,
}
