use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Error {
    Database(String),
    Spotify(String),
    Decode(String),
    Encode(String),
    WebSocket(String),
    Forbidden(String),
    FileSystem(String),
    InvalidRequest(String),
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
