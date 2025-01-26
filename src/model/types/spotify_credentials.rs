use rspotify::Credentials;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpotifyCredentials {
    pub id: String,
    pub secret: String,
}

impl From<Credentials> for SpotifyCredentials {
    fn from(credentials: Credentials) -> Self {
        Self {
            id: credentials.id,
            secret: credentials.secret.unwrap(),
        }
    }
}
