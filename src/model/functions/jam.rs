use crate::model::types::*;
use leptos::{ev::play, logging::*, server_fn::redirect};
use rand::seq::SliceRandom;
use real_time::Changed;
use rspotify::Credentials;
use sqlx::PgPool;

use super::notify;

pub async fn get_jam(jam_id: &str, pool: &sqlx::PgPool) -> Result<Jam, sqlx::Error> {
    let jam = sqlx::query!("SELECT * FROM jams WHERE id = $1", jam_id.to_lowercase())
        .fetch_one(pool)
        .await?;
    Ok(Jam {
        id: jam.id,
        name: jam.name,
        max_song_count: jam.max_song_count as u8,
    })
}

pub async fn create_host(
    code: String,
    host_id: String,
    spotify_credentials: &SpotifyCredentials,
    reqwest_client: &reqwest::Client,
    pool: &sqlx::PgPool,
    redirect_uri: &str,
) -> Result<(), Error> {
    use http::StatusCode;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    let body = {
        let mut body = HashMap::new();
        body.insert("code", code.as_str());
        body.insert("redirect_uri", redirect_uri);
        body.insert("grant_type", "authorization_code");
        body.insert("client_id", &spotify_credentials.id);
        body.insert("client_secret", &spotify_credentials.secret);
        body
    };
    let client = reqwest_client;
    let res = match client
        .post("https://accounts.spotify.com/api/token")
        .form(&body)
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return Err(Error::Spotify(format!(
                "error while acquiring temp spotify token: {:#?}",
                e
            )))
        }
    };

    #[derive(Serialize, Deserialize, Debug)]
    struct AccessToken {
        access_token: String,
        token_type: String,
        scope: String,
        expires_in: i64,
        refresh_token: String,
    }

    let res = match &res.status() {
        &StatusCode::OK | &StatusCode::CREATED => res.text().await,
        _ => {
            eprintln!("Error: {:?}", res);
            sqlx::query!("DELETE FROM hosts WHERE id = $1", host_id)
                .execute(pool)
                .await?;
            return Err(Error::Database(format!(
                "error while acquiring spotify token, spotify returned not ok response code: {:#?}",
                res
            )));
        }
    };

    let res = match res {
        Ok(res) => res,
        Err(e) => {
            return Err(Error::Decode(format!(
                "error while getting text from spotify response: {:#?}",
                e
            )))
        }
    };

    let token: AccessToken = match serde_json::from_str(res.as_str()) {
        Ok(token) => token,
        Err(e) => {
            return Err(Error::Decode(format!(
                "error while deserializing spotify response spotify token: {:#?}",
                e
            )))
        }
    };

    let now = chrono::Utc::now().timestamp();
    let expires_at = now + token.expires_in;

    let access_token_id = cuid2::create_id();
    sqlx::query!(
        "INSERT INTO access_tokens 
            (access_token, expires_at, scope, refresh_token,id, host_id) 
        VALUES 
            ($1, $2, $3, $4,$5,$6)",
        token.access_token,
        expires_at,
        token.scope,
        token.refresh_token,
        access_token_id,
        host_id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_jam(
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<real_time::Changed, sqlx::Error> {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    sqlx::query!("DELETE FROM jams WHERE id = $1", jam_id)
        .execute(pool)
        .await?;
    Ok(real_time::Changed::new().ended())
}

pub async fn create_jam(
    name: &str,
    host_id: &str,
    max_song_count: i16,
    pool: &sqlx::PgPool,
    spotify_credentials: SpotifyCredentials,
) -> Result<JamId, Error> {
    let jam_id = cuid2::CuidConstructor::new()
        .with_length(6)
        .create_id()
        .to_lowercase();

    let mut transaction = pool.begin().await?;

    let error = sqlx::query!(
        "INSERT INTO jams (id, max_song_count, host_id, name) VALUES ($1, $2, $3, $4)",
        &jam_id,
        &max_song_count,
        host_id,
        name
    )
    .execute(&mut *transaction)
    .await;
    match error {
        Ok(_) => (),
        Err(sqlx::Error::Database(e)) => {
            if e.message() == "duplicate key value violates unique constraint \"jams_host_id_key\""
            {
                let jam_id = sqlx::query!("SELECT id FROM jams WHERE host_id=$1", host_id)
                    .fetch_one(pool)
                    .await?
                    .id;
                return Err(Error::HostAlreadyInJam { jam_id });
            }
        }
        Err(e) => {
            return Err(e.into());
        }
    }

    sqlx::query!(
        "INSERT INTO users (id, jam_id, name) VALUES ($1, $1, $2)",
        jam_id,
        name
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(jam_id)
}

pub async fn set_current_song_position(
    jam_id: &str,
    percentage: f32,
    pool: &sqlx::PgPool,
) -> Result<real_time::Changed, Error> {
    if !(0.0..=1.0).contains(&percentage) {
        return Err(Error::InvalidRequest(
            "Percentage must be between 0 and 1".to_string(),
        ));
    }
    sqlx::query!(
        "UPDATE jams SET song_position = $1 WHERE id = $2",
        percentage,
        jam_id
    )
    .execute(pool)
    .await?;
    Ok(real_time::Changed::new().position())
}

pub async fn get_current_song_position(jam_id: &str, pool: &sqlx::PgPool) -> Result<f32, Error> {
    let row = sqlx::query!("SELECT song_position FROM jams WHERE id = $1", jam_id)
        .fetch_one(pool)
        .await?;
    Ok(row.song_position)
}

pub async fn get_current_song(
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<Option<Song>, sqlx::Error> {
    struct SongDb {
        pub id: String,
        pub spotify_id: String,
        pub user_id: String,
        pub name: String,
        pub album: String,
        pub duration: i32,
        pub artists: Option<Vec<String>>,
        pub image_url: String,
    }

    let song = sqlx::query_as!(SongDb, "SELECT * FROM songs WHERE user_id=$1", jam_id)
        .fetch_optional(pool)
        .await?;

    let song = match song {
        Some(song) => song,
        None => return Ok(None),
    };

    Ok(Some(Song {
        votes: Vote {
            votes: 0,
            have_you_voted: None,
        },
        spotify_id: song.spotify_id,
        id: Some(song.id),
        user_id: Some(song.user_id),
        name: song.name,
        artists: song
            .artists
            .unwrap_or(vec!["no artist found in cache, this is a bug".to_string()]),
        album: song.album,
        duration: song.duration as u32,
        image_url: song.image_url,
    }))
}

/// doesn't need to have the song id as some, it will generate a new one, either way
pub async fn set_current_song(
    song: &Song,
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<real_time::Changed, Error> {
    sqlx::query!("DELETE FROM songs WHERE user_id=$1", jam_id)
        .execute(pool)
        .await?;
    let song_id = cuid2::create_id();

    sqlx::query!(
        "INSERT INTO songs (id, user_id, name, album, duration, artists, image_url, spotify_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        song_id,
        jam_id,
        song.name,
        song.album,
        song.duration as i32,
        &song.artists,
        song.image_url,
        song.spotify_id
    )
    .execute(pool)
    .await?;
    Ok(real_time::Changed::new().current_song())
}

pub async fn dose_jam_exist(jam_id: &str, pool: &sqlx::PgPool) -> Result<bool, Error> {
    sqlx::query!("SELECT EXISTS(SELECT 1 FROM jams WHERE id=$1)", jam_id)
        .fetch_one(pool)
        .await
        .map(|b| b.exists.unwrap_or(false))
        .map_err(|e| e.into())
}

pub async fn next_song(
    jam_id: String,
    pool: &PgPool,
    credentials: SpotifyCredentials,
) -> Result<Changed, Error> {
    use super::*;

    let id = Id::new(IdType::General, jam_id);

    let mut changed = Changed::new();
    let top_song = get_top_song(pool, id.jam_id.clone()).await?;

    let top_song = match top_song {
        Some(song) => Some(song),
        None => match get_next_song_from_player(id.jam_id(), pool, credentials.clone()).await {
            Ok(song) => match song {
                Some(song) => Some(song),
                None => None,
            },
            Err(e) => {
                let songs = search(
                    "Never gonna give you up",
                    pool,
                    &id.jam_id,
                    credentials.clone(),
                )
                .await?
                .remove(0);
                Some(songs)
            }
        },
    };

    if let Some(song) = top_song {
        changed = changed.merge_with_other(set_current_song(&song, id.jam_id(), pool).await?);
        changed = changed.merge_with_other(reset_votes(id.jam_id(), pool).await?);

        play_song(&song.spotify_id, id.jam_id(), pool, credentials).await?;
    };

    Ok(changed)
}
