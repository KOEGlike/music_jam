use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Error from database: {0}")]
    Database(String),
    #[error("Error from spotify: {0}")]
    Spotify(String),
    #[error("Error from serde decode: {0}")]
    Decode(String),
    #[error("Error from serde encode: {0}")]
    Encode(String),
    #[error("Error with web socket")]
    WebSocket(String),
    #[error("This action is not allowed for you: {0}")]
    Forbidden(String),
    #[error("There is something missing or something that is not allow with the file system: {0}")]
    FileSystem(String),
    #[error("Your request is incorrect: {0}")]
    InvalidRequest(String),
    #[error("The host cant create another jam cuz he is in one already, jam id: {jam_id}")]
    HostAlreadyInJam { jam_id: String },
    #[error("This user has reached |insert pronoun here| song limit")]
    UserHasTooTheMaxSongAmount,
    #[error("A env was not found: {0}")]
    EnvNotFound(String),
    #[error("song already in jam")]
    SongAlreadyInJam,
    #[error("A entry was not found: {0}")]
    DoesNotExist(String),
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
            Error::FileSystem(_) => 4500,
            Error::InvalidRequest(_) => 4400,
            Error::HostAlreadyInJam { .. } => 4400,
            Error::UserHasTooTheMaxSongAmount => 4400,
            Error::EnvNotFound(_) => 4500,
            Error::SongAlreadyInJam => 4400,
            Error::DoesNotExist(_) => 4404,
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
            Error::FileSystem(s) => s,
            Error::InvalidRequest(s) => s,
            Error::HostAlreadyInJam { jam_id } => {
                format!("Host is already in jam with id: {}", jam_id)
            }
            Error::UserHasTooTheMaxSongAmount => "User has too the max song amount".to_string(),
            Error::EnvNotFound(s) => s,
            Error::SongAlreadyInJam => "Song already in jam".to_string(),
            Error::DoesNotExist(s) => s,
        }
    }
}

use rspotify::model::idtypes::IdError;
use rspotify::ClientError;

impl From<ClientError> for Error {
    fn from(e: ClientError) -> Self {
        Error::Spotify(format!("client error: {:?}", e))
    }
}

impl From<IdError> for Error {
    fn from(e: IdError) -> Self {
        Error::Spotify(format!("id error: {:?}", e))
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
        if let sqlx::Error::RowNotFound = e {
            return Error::DoesNotExist(format!("sqlx error: {:?}", e.to_string()));
        }
        Error::Database(format!("sqlx error: {:?}", e.to_string()))
    }
}

#[cfg(feature = "ssr")]
impl From<std::env::VarError> for Error {
    fn from(value: std::env::VarError) -> Self {
        Error::EnvNotFound(value.to_string())
    }
}
