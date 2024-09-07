use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Vote {
    pub votes: u64,
    ///none if requested by the host, or a unknown person
    pub have_you_voted: Option<bool>,
}

pub type Votes = HashMap<String, Vote>;