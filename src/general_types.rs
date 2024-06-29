#[cfg(feature = "ssr")]
use axum::extract::FromRef;
use nestify::nest;
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
    pub album_url: String,
    pub votes: i32,
}

pub mod real_time {

    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Update {
        Users(Vec<User>),
        Songs(Vec<Song>),
        Error(Error),
        Vote { song_id: String, vote_nr: u16 },
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Request {
        RemoveUser { user_id: String },
        AddSong { song_id: String },
        RemoveSong { song_id: String },
        AddVote { song_id: String },
        RemoveVote { song_id: String },
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Error {
        Database(String),
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
            }
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
}
