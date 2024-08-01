use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Changed {
    pub users: bool,
    ///this has to be re-fetched on the clients WS thread because every user can vote on songs and thus has a different result of for songs
    pub songs: bool,
    /// this has to be re-fetched on the clients WS thread because every user can vote on songs and thus has a different result of for votes
    pub votes: bool,
    pub ended: bool,
    pub position: bool,
    pub current_song: bool,
}

impl Changed {
    pub fn new() -> Self {
        Self {
            users: false,
            songs: false,
            votes: false,
            ended: false,
            position: false,
            current_song: false,
        }
    }

    pub fn merge_with_other(self, other: Self) -> Self {
        Self {
            users: self.users || other.users,
            songs: self.songs || other.songs,
            votes: self.votes || other.votes,
            ended: self.ended || other.ended,
            position: self.position || other.position,
            current_song: self.current_song || other.current_song,
        }
    }

    pub fn users(self) -> Self {
        Self {
            users: true,
            ..self
        }
    }

    pub fn songs(self) -> Self {
        Self {
            songs: true,
            ..self
        }
    }

    pub fn votes(self) -> Self {
        Self {
            votes: true,
            ..self
        }
    }

    
    

    pub fn ended(self) -> Self {
        Self {
            ended: true,
            ..self
        }
    }

    pub fn position(self) -> Self {
        Self {
            position: true,
            ..self
        }
    }

    pub fn current_song(self) -> Self {
        Self {
            current_song: true,
            ..self
        }
    }


    /// This function sets all the fields to true except for ended
    pub fn all() -> Self {
        Self {
            users: true,
            songs: true,
            votes: true,
            ended: false,
            position: true,
            current_song: true,
        }
    }
}