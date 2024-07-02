#[cfg(feature = "ssr")]
use axum::extract::FromRef;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpotifyCredentials {
    pub id: String,
    pub secret: String,
}

#[cfg(feature = "ssr")]
#[derive(FromRef, Clone, Debug)]
pub struct AppState {
    pub db: Db,
    pub reqwest_client: reqwest::Client,
    pub spotify_credentials: SpotifyCredentials,
    pub leptos_options: leptos::LeptosOptions,
}

#[cfg(feature = "ssr")]
#[derive(Clone, Debug)]
pub struct Db {
    pub pool: sqlx::PgPool,
    pub url: String,
}

#[cfg(feature = "ssr")]
impl AppState {
    pub async fn new(leptos_options: leptos::LeptosOptions) -> Result<Self, Box<dyn Error>> {
        dotenvy::dotenv().unwrap();
        let reqwest_client = reqwest::Client::new();
        let spotify_id = std::env::var("SPOTIFY_ID")?;
        let spotify_secret = std::env::var("SPOTIFY_SECRET")?;
        let db_url = std::env::var("DATABASE_URL")?;
        let db_pool = sqlx::PgPool::connect(&db_url).await?;

        let spotify_credentials = SpotifyCredentials {
            id: spotify_id,
            secret: spotify_secret,
        };

        Ok(Self {
            db: Db {
                pool: db_pool,
                url: db_url,
            },
            reqwest_client,
            spotify_credentials,
            leptos_options,
        })
    }
}

pub type JamId = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub jam_id: String,
    pub name: String,
    pub pfp_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Jam {
    pub id: String,
    pub name: String,
    pub max_song_count: i8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub artist: String,
    pub album: String,
    pub duration: i32,
    pub image_url: String,
    pub votes: i64,
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


use std::collections::HashMap;
pub type Votes = HashMap<String, i64>;

pub mod real_time {
    use super::*;

    pub enum Channels {
        Users,
        Songs,
        Votes,
    }

    impl From<Channels> for String {
        fn from(val: Channels) -> Self {
            match val {
                Channels::Users => "users".to_string(),
                Channels::Songs => "songs".to_string(),
                Channels::Votes => "votes".to_string(),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Update {
        Users(Vec<User>),
        Songs(Vec<Song>),
        Error(Error),
        Votes(Votes),
    }

    impl From<Votes> for Update {
        fn from(votes: Votes) -> Self {
            Update::Votes(votes)
        }
    }
    impl From<Vec<Song>> for Update {
        fn from(songs: Vec<Song>) -> Self {
            Update::Songs(songs)
        }
    }
    impl From<Vec<User>> for Update {
        fn from(users: Vec<User>) -> Self {
            Update::Users(users)
        }
    }
    impl From<Error> for Update {
        fn from(e: Error) -> Self {
            Update::Error(e)
        }
    }
    #[cfg(feature = "ssr")]
    impl<T: Into<Update>> From<Result<T, sqlx::Error>> for Update {
        fn from(res: Result<T, sqlx::Error>) -> Self {
            match res {
                Ok(val) => val.into(),
                Err(e) => Update::Error(Error::Database(e.to_string())),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Request {
        KickUser { user_id: String },
        AddSong { song_id: String },
        RemoveSong { song_id: String },
        AddVote { song_id: String },
        RemoveVote { song_id: String },
        Update,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Error {
        Database(String),
        Spotify(String),
        Decode(String),
        Encode(String),
        WebSocket(String),
        Forbidden(String),
    }

    impl Error {
        pub fn to_code(&self) -> u16 {
            match *self {
                Error::Database(_) => 4500,
                Error::Decode(_) => 4500,
                Error::Encode(_) => 4500,
                Error::WebSocket(_) => 4500,
                Error::Forbidden(_) => 4403,
                Error::Spotify(_) => 4500,
                
            }
        }
    }

    impl From<Error> for String {
        fn from(val: Error) -> Self {
            match val {
                Error::Database(s) => s,
                Error::Decode(s) => s,
                Error::Encode(s) => s,
                Error::WebSocket(s) => s,
                Error::Forbidden(s) => s,
                Error::Spotify(s) => s,
            }
        }
    }

    use rspotify::model::idtypes::IdError;
    use rspotify::ClientError;

    impl From<ClientError> for Error {
        fn from(e: ClientError) -> Self {
            Error::Spotify(e.to_string())
        }
    }

    impl From<IdError> for Error {
        fn from(e: IdError) -> Self {
            Error::Spotify(e.to_string())
        }
    }

    #[cfg(feature = "ssr")]
    use axum::extract::ws::CloseFrame;
    #[cfg(feature = "ssr")]
    impl Error {
        pub fn to_close_frame(self) -> CloseFrame<'static> {
            use std::borrow::Cow;

            let code = self.to_code();
            let message: String = self.into();
            CloseFrame {
                code,
                reason: Cow::Owned(message),
            }
        }
    }

    #[cfg(feature = "ssr")]
    impl From<sqlx::Error> for Error {
        fn from(e: sqlx::Error) -> Self {
            Error::Database(e.to_string())
        }
    }
}
