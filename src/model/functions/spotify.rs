use crate::model::types::*;
use leptos::logging::*;
use rspotify::{
    clients::{BaseClient, OAuthClient},
    model::{AdditionalType, Id, SearchResult, TrackId},
    AuthCodeSpotify,
};

pub async fn switch_playback_to_device<'e>(
    device_id: &str,
    jam_id: &str,
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    credentials: SpotifyCredentials,
) -> Result<(), Error> {
    let token = get_access_token(transaction, jam_id, credentials).await?;
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

async fn get_maybe_expired_access_token<'e>(
    executor: impl sqlx::PgExecutor<'e>,
    jam_id: &str,
) -> Result<rspotify::Token, sqlx::Error> {
    let token = sqlx::query_as!(
        AccessTokenDb,
        "SELECT * FROM access_tokens WHERE host_id=(SELECT host_id FROM jams WHERE id=$1) ",
        jam_id
    )
    .fetch_one(executor)
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
pub async fn get_access_token<'e>(
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    jam_id: &str,
    credentials: SpotifyCredentials,
) -> Result<rspotify::Token, Error> {
    let token = get_maybe_expired_access_token(&mut **transaction, jam_id).await?;
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
    .execute(&mut **transaction)
    .await?;

    log!("updated token");

    Ok(new_token)
}

pub async fn search<'e>(
    query: &str,
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    jam_id: &str,
    credentials: SpotifyCredentials,
) -> Result<Vec<Song>, Error> {
    use rspotify::prelude::*;
    use rspotify::AuthCodeSpotify;

    if query.is_empty() {
        return Ok(vec![]);
    }

    let token = get_access_token(transaction, jam_id, credentials).await?;
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
        return Err(Error::Spotify(
            "Error in search, returned other then tracks".to_string(),
        ));
    };

    let songs_in_jam = sqlx::query!(
        "SELECT id FROM songs WHERE user_id IN (SELECT id FROM users WHERE jam_id=$1);",
        jam_id
    )
    .fetch_all(&mut **transaction)
    .await?
    .into_iter()
    .map(|song| song.id)
    .collect::<Vec<String>>();

    let songs = songs
        .items
        .into_iter()
        .filter(|song| !songs_in_jam.contains(&song.id.as_ref().unwrap().id().to_owned()))
        .collect::<Vec<_>>();

    let songs = songs.into_iter().map(track_to_song).collect::<Vec<Song>>();

    Ok(songs)
}

pub async fn get_current_song_from_player<'e>(
    jam_id: &str,
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    credentials: SpotifyCredentials,
) -> Result<Option<Song>, Error> {
    let token = get_access_token(transaction, jam_id, credentials).await?;
    let client = AuthCodeSpotify::from_token(token);
    let current = client.current_playing(None, None::<Vec<_>>).await?;
    let current = match current {
        Some(song) => song,
        None => return Ok(None),
    };
    let current = match current.item {
        Some(rspotify::model::PlayableItem::Track(track)) => track,
        _ => return Ok(None),
    };
    Ok(Some(track_to_song(current)))
}

pub async fn get_next_song_from_player<'e>(
    jam_id: &str,
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    credentials: SpotifyCredentials,
) -> Result<Option<Song>, Error> {
    let token = get_access_token(transaction, jam_id, credentials).await?;
    let client = AuthCodeSpotify::from_token(token);
    let current = client.current_user_queue().await?.queue.into_iter().next();
    let current = match current {
        Some(song) => song,
        None => return Ok(None),
    };
    let current = match current {
        rspotify::model::PlayableItem::Track(track) => track,
        _ => return Ok(None),
    };
    Ok(Some(track_to_song(current)))
}

pub fn track_to_song(track: rspotify::model::FullTrack) -> Song {
    Song {
        id: None,
        spotify_id: track
            .id
            .map(|id| id.id().to_string())
            .unwrap_or("no id, wtf".to_string()),
        user_id: None,
        name: track.name,
        artists: track.artists.into_iter().map(|a| a.name).collect(),
        album: track.album.name,
        duration: track.duration.num_seconds() as u32,
        image_url: track
            .album
            .images
            .into_iter()
            .next()
            .map(|i| i.url)
            .unwrap_or_default(),
        votes: Vote {
            votes: 0,
            have_you_voted: None,
        },
    }
}

pub async fn play_song<'e>(
    spotify_song_id: &str,
    jam_id: &str,
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    credentials: SpotifyCredentials,
) -> Result<(), Error> {
    let token = get_access_token(transaction, jam_id, credentials).await?;
    let client = AuthCodeSpotify::from_token(token);
    let song_id = match TrackId::from_id(spotify_song_id) {
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
