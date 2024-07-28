use crate::general::types::*;
use leptos::logging::*;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{SearchResult, TrackId},
    AuthCodeSpotify,
};



pub async fn switch_playback_to_device(
    device_id: &str,
    jam_id: &str,
    pool: &sqlx::PgPool,
    credentials: SpotifyCredentials,
) -> Result<(), Error> {
    let token = get_access_token(pool, jam_id, credentials).await?;
    let client = AuthCodeSpotify::from_token(token);
    if let Err(e) = client.transfer_playback(device_id, Some(true)).await {
        return Err(Error::Spotify(format!(
            "could not switch playback to device: {}",
            e
        )));
    };
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct AccessTokenDb {
    pub refresh_token: String,
    pub access_token: String,
    pub expires_at: i64,
    pub scope: String,
    pub id: String,
    pub host_id: String,
}

async fn get_maybe_expired_access_token(
    pool: &sqlx::PgPool,
    jam_id: &str,
) -> Result<rspotify::Token, sqlx::Error> {
    let token=sqlx::query_as!(
        AccessTokenDb,
        "SELECT * FROM access_tokens WHERE id=(SELECT access_token FROM hosts WHERE id=(SELECT host_id FROM jams WHERE id=$1))",
        jam_id
    )
    .fetch_one(pool)
    .await?;

    let expires_at = chrono::DateTime::from_timestamp(token.expires_at, 0).unwrap();
    let expires_at = Some(expires_at);
    let expires_in = token.expires_at - chrono::Utc::now().timestamp();
    let expires_in = chrono::TimeDelta::new(expires_in, 0).unwrap();

    let token = rspotify::Token {
        access_token: token.access_token,
        expires_in,
        expires_at,
        refresh_token: Some(token.refresh_token),
        scopes: rspotify::scopes!(token.scope),
    };

    Ok(token)
}

///this also refreshes the token if it is expired
pub async fn get_access_token(
    pool: &sqlx::PgPool,
    jam_id: &str,
    credentials: SpotifyCredentials,
) -> Result<rspotify::Token, Error> {
    let token = get_maybe_expired_access_token(pool, jam_id).await?;
    let now = chrono::Utc::now().timestamp();
    if now < token.expires_at.unwrap_or_default().timestamp() {
        return Ok(token);
    }
    let old_access_token = token.access_token.clone();
    let client = rspotify::AuthCodeSpotify::from_token_with_config(
        token,
        rspotify::Credentials {
            id: credentials.id,
            secret: Some(credentials.secret),
        },
        rspotify::OAuth::default(),
        rspotify::Config::default(),
    );
    client.refetch_token().await?;
    client.refresh_token().await?;
    let new_token = client
        .get_token()
        .as_ref()
        .lock()
        .await
        .unwrap()
        .clone()
        .unwrap();

    sqlx::query!(
        "UPDATE access_tokens SET access_token=$1, expires_at=$2, scope=$3, refresh_token=$4 WHERE access_token=$5;",
        new_token.access_token,
        now + new_token.expires_in.num_seconds(),
        new_token.scopes.clone().into_iter().collect::<Vec<_>>().join(" "),
        new_token.refresh_token,
        old_access_token
    )
    .execute(pool)
    .await?;

    log!("updated token");

    Ok(new_token)
}

pub async fn search(
    query: &str,
    pool: &sqlx::PgPool,
    jam_id: &str,
    credentials: SpotifyCredentials,
) -> Result<Vec<Song>, Error> {
    use rspotify::prelude::*;
    use rspotify::AuthCodeSpotify;

    let token = get_access_token(pool, jam_id, credentials).await?;
    let client = AuthCodeSpotify::from_token(token);
    let result = client
        .search(
            query,
            rspotify::model::SearchType::Track,
            None,
            None,
            Some(30),
            Some(0),
        )
        .await?;
    let songs = if let SearchResult::Tracks(tracks) = result {
        tracks
    } else {
        return Err(Error::Spotify("Error in search".to_string()));
    };

    let songs_in_jam = sqlx::query!(
        "SELECT id FROM songs WHERE user_id IN (SELECT id FROM users WHERE jam_id=$1);",
        jam_id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|song| song.id)
    .collect::<Vec<String>>();

    let songs = songs
        .items
        .into_iter()
        .filter(|song| !songs_in_jam.contains(&song.id.as_ref().unwrap().id().to_owned()))
        .collect::<Vec<_>>();

    let songs = songs
        .iter()
        .map(|track| {
            let id = track.id.as_ref().unwrap().id().to_owned();
            Song {
                id,
                user_id: None,
                name: track.name.clone(),
                artists: track.artists.iter().map(|a| a.name.clone()).collect(),
                album: track.album.name.clone(),
                duration: track.duration.num_seconds() as u16,
                image_url: track.album.images[0].url.clone(),
                votes: Vote {
                    votes: 0,
                    have_you_voted: None,
                },
            }
        })
        .collect::<Vec<Song>>();

    Ok(songs)
}

pub async fn play_song(
    song_id: &str,
    jam_id: &str,
    pool: &sqlx::PgPool,
    credentials: SpotifyCredentials,
) -> Result<(), Error> {
    let token = get_access_token(pool, jam_id, credentials).await?;
    let client = AuthCodeSpotify::from_token(token);
    let song_id = match TrackId::from_id(song_id) {
        Ok(id) => id,
        Err(e) => {
            return Err(Error::Spotify(format!(
                "could not play song, song id is not correct: {}",
                e
            )))
        }
    };
    if let Err(e) = client
        .add_item_to_queue(rspotify::model::PlayableId::Track(song_id), None)
        .await
    {
        return Err(Error::Spotify(format!(
            "could not play song, could add song to queue: {}",
            e
        )));
    };
    if let Err(e) = client.next_track(None).await {
        return Err(Error::Spotify(format!(
            "could not play song, could not skip to next song: {}",
            e
        )));
    };
    Ok(())
}
