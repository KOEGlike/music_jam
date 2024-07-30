use crate::general::functions;
use crate::general::types::*;
use serde::{Deserialize, Serialize};

/// The update that is sent to the client
/// if the field is some then it was updated
/// if the field is none then it was not updated
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Update {
    pub users: Option<Vec<User>>,
    pub songs: Option<Vec<Song>>,
    pub error: Vec<Error>,
    pub votes: Option<Votes>,
    pub search: Option<Vec<Song>>,
    pub ended: Option<()>,
    /// the percentage of the current song
    pub position: Option<f32>,
    /// the current song may be null, so there is an option inside an option
    pub current_song: Option<Option<Song>>,
}

impl Update {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn users(self, users: Vec<User>) -> Self {
        Self {
            users: Some(users),
            ..self
        }
    }

    ///only the jam id id used from the id
    pub async fn users_from_jam(self, id: &IdType, pool: &sqlx::PgPool) -> Self {
        match functions::get_users(pool, id).await {
            Ok(users) => self.users(users),
            Err(e) => self.error(e.into()),
        }
    }

    pub fn songs(self, songs: Vec<Song>) -> Self {
        Self {
            songs: Some(songs),
            ..self
        }
    }

    pub async fn songs_from_jam(self, id: &IdType, pool: &sqlx::PgPool) -> Self {
        match functions::get_songs(pool, id).await {
            Ok(songs) => self.songs(songs),
            Err(e) => self.error(e.into()),
        }
    }

    pub fn error(mut self, error: Error) -> Self {
        self.error.push(error);
        self
    }

    pub fn votes(self, votes: Votes) -> Self {
        Self {
            votes: Some(votes),
            ..self
        }
    }

    pub async fn votes_from_jam(self, id: &IdType, pool: &sqlx::PgPool) -> Self {
        match functions::get_votes(pool, id).await {
            Ok(votes) => self.votes(votes),
            Err(e) => self.error(e.into()),
        }
    }

    pub fn search(self, search: Vec<Song>) -> Self {
        Self {
            search: Some(search),
            ..self
        }
    }

    pub fn ended(self) -> Self {
        Self {
            ended: Some(()),
            ..self
        }
    }

    pub fn position(self, percentage: f32) -> Self {
        Self {
            position: Some(percentage),
            ..self
        }
    }

    pub async fn position_from_jam(self, jam_id: &str, pool: &sqlx::PgPool) -> Self {
        match functions::get_current_song_position(jam_id, pool).await {
            Ok(percentage) => self.position(percentage),
            Err(e) => self.error(e.into()),
        }
    }

    pub fn current_song(self, song: Option<Song>) -> Self {
        Self {
            current_song: Some(song),
            ..self
        }
    }

    pub async fn current_song_from_jam(self, jam_id: &str, pool: &sqlx::PgPool) -> Self {
        match functions::get_current_song(jam_id, pool).await {
            Ok(song) => self.current_song(song),
            Err(e) => self.error(e.into()),
        }
    }

    pub fn merge_with_other(self, other: Self) -> Self {
        Self {
            users: other.users.or(self.users),
            songs: other.songs.or(self.songs),
            error: self
                .error
                .into_iter()
                .chain(other.error.into_iter())
                .collect(),
            votes: other.votes.or(self.votes),
            search: other.search.or(self.search),
            ended: other.ended.or(self.ended),
            position: other.position.or(self.position),
            current_song: other.current_song.or(self.current_song),
        }
    }

    pub async fn from_changed(changed:real_time::Changed, id: &IdType, pool: &sqlx::PgPool) -> Self {
        let mut update = Update::new();
        if changed.users {
            update = update.users_from_jam(id, pool).await;
        }
        if changed.songs {
            update = update.songs_from_jam(id, pool).await;
        }
        if changed.votes {
            update = update.votes_from_jam(id, pool).await;
        }
        if changed.ended {
            update = update.ended();
        }
        if changed.position {
            update = update.position_from_jam(id.jam_id(), pool).await;
        }
        if changed.current_song {
            update = update.current_song_from_jam(id.jam_id(), pool).await;
        }
        update
    }

}


impl From<Votes> for Update {
    fn from(votes: Votes) -> Self {
        Update::new().votes(votes)
    }
}

impl From<Vec<Song>> for Update {
    fn from(songs: Vec<Song>) -> Self {
        Update::new().songs(songs)
    }
}

impl From<Vec<User>> for Update {
    fn from(users: Vec<User>) -> Self {
        Update::new().users(users)
    }
}

impl From<Error> for Update {
    fn from(e: Error) -> Self {
        Update::new().error(e)
    }
}

#[cfg(feature = "ssr")]
impl<T: Into<Update>> From<Result<T, sqlx::Error>> for Update {
    fn from(res: Result<T, sqlx::Error>) -> Self {
        match res {
            Ok(val) => val.into(),
            Err(e) => Update::new().error(Error::Database(e.to_string())),
        }
    }
}
