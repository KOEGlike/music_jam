use super::SearchResult;
#[cfg(feature = "ssr")]
use crate::model::functions;
use crate::model::types::*;
use serde::{Deserialize, Serialize};

/// The update that is sent to the client
/// if the field is some then it was updated
/// if the field is none then it was not updated
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Update {
    pub users: Option<Vec<User>>,
    pub songs: Option<Vec<Song>>,
    pub errors: Vec<Error>,
    pub votes: Option<Votes>,
    pub search: Option<SearchResult>,
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

    #[cfg(feature = "ssr")]
    pub async fn users_from_jam<'e>(self, id: &Id, executor: impl sqlx::PgExecutor<'e>) -> Self {
        match functions::get_users(executor, id).await {
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

    #[cfg(feature = "ssr")]
    pub async fn songs_from_jam<'e>(
        self,
        id: &Id,
        transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    ) -> Self {
        match functions::get_songs(transaction, id).await {
            Ok(songs) => self.songs(songs),
            Err(e) => self.error(e.into()),
        }
    }

    pub fn error(mut self, error: Error) -> Self {
        self.errors.push(error);
        self
    }

    pub fn error_vec(mut self, errors: Vec<Error>) -> Self {
        self.errors.extend(errors);
        self
    }

    pub fn votes(self, votes: Votes) -> Self {
        Self {
            votes: Some(votes),
            ..self
        }
    }

    #[cfg(feature = "ssr")]
    pub async fn votes_from_jam<'e>(
        self,
        id: &Id,
        transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    ) -> Self {
        match functions::get_votes(transaction, id).await {
            Ok(votes) => self.votes(votes),
            Err(e) => self.error(e.into()),
        }
    }

    pub fn search(self, search: SearchResult) -> Self {
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

    #[cfg(feature = "ssr")]
    pub async fn position_from_jam<'e>(
        self,
        jam_id: &str,
        executor: impl sqlx::PgExecutor<'e>,
    ) -> Self {
        match functions::get_current_song_position(jam_id, executor).await {
            Ok(percentage) => self.position(percentage),
            Err(e) => self.error(e),
        }
    }

    pub fn current_song(self, song: Option<Song>) -> Self {
        Self {
            current_song: Some(song),
            ..self
        }
    }

    #[cfg(feature = "ssr")]
    pub async fn current_song_from_jam<'e>(
        self,
        jam_id: &str,
        executor: impl sqlx::PgExecutor<'e>,
    ) -> Self {
        match crate::model::functions::get_current_song(jam_id, executor).await {
            Ok(song) => self.current_song(song),
            Err(e) => self.error(e.into()),
        }
    }

    pub fn merge_with_other(self, other: Self, prioritize_other: bool) -> Self {
        if prioritize_other {
            Self {
                users: other.users.or(self.users),
                songs: other.songs.or(self.songs),
                errors: self.errors.into_iter().chain(other.errors).collect(),
                votes: other.votes.or(self.votes),
                search: other.search.or(self.search),
                ended: other.ended.or(self.ended),
                position: other.position.or(self.position),
                current_song: other.current_song.or(self.current_song),
            }
        } else {
            Self {
                users: self.users.or(other.users),
                songs: self.songs.or(other.songs),
                errors: self.errors.into_iter().chain(other.errors).collect(),
                votes: self.votes.or(other.votes),
                search: self.search.or(other.search),
                ended: self.ended.or(other.ended),
                position: self.position.or(other.position),
                current_song: self.current_song.or(other.current_song),
            }
        }
    }

    #[cfg(feature = "ssr")]
    pub async fn from_changed<'e>(
        changed: real_time::Changed,
        id: &Id,
        transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    ) -> Self {
        use tokio::sync::Mutex;

        let update = Update::new();
        let transaction = Mutex::new(transaction);

        let users_future = async {
            if changed.users {
                let mut transaction = transaction.lock().await;
                update.clone().users_from_jam(id, &mut ***transaction).await
            } else {
                update.clone()
            }
        };

        let songs_future = async {
            if changed.songs {
                let mut transaction = transaction.lock().await;
                update.clone().songs_from_jam(id, *transaction).await
            } else {
                update.clone()
            }
        };

        let votes_future = async {
            if changed.votes {
                let mut transaction = transaction.lock().await;
                update.clone().votes_from_jam(id, &mut **transaction).await
            } else {
                update.clone()
            }
        };

        let ended_future = async {
            if changed.ended {
                update.clone().ended()
            } else {
                update.clone()
            }
        };

        let position_future = async {
            if changed.position {
                let mut transaction = transaction.lock().await;
                update
                    .clone()
                    .position_from_jam(id.jam_id(), &mut ***transaction)
                    .await
            } else {
                update.clone()
            }
        };

        let current_song_future = async {
            if changed.current_song {
                let mut transaction = transaction.lock().await;
                update
                    .clone()
                    .current_song_from_jam(id.jam_id(), &mut ***transaction)
                    .await
            } else {
                update.clone()
            }
        };

        let (
            users_update,
            songs_update,
            votes_update,
            ended_update,
            position_update,
            current_song_update,
        ) = tokio::join!(
            users_future,
            songs_future,
            votes_future,
            ended_future,
            position_future,
            current_song_future
        );

        update
            .merge_with_other(users_update, false)
            .merge_with_other(songs_update, false)
            .merge_with_other(votes_update, false)
            .merge_with_other(ended_update, false)
            .merge_with_other(position_update, false)
            .merge_with_other(current_song_update, false)
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
