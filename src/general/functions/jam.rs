use crate::general::types::*;
use leptos::logging::*;

use super::notify;

pub async fn get_jam(jam_id: &str, pool: &sqlx::PgPool) -> Result<Jam, sqlx::Error> {
    let jam = sqlx::query!("SELECT * FROM jams WHERE id = $1", jam_id)
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
) -> Result<(), Error> {
    use http::StatusCode;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    let body = {
        let mut body = HashMap::new();
        body.insert("code", code.as_str());
        body.insert("redirect_uri", "http://localhost:3000/create-host");
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
            log!("Error: {:?}", res);
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
) -> Result<JamId, Error> {
    let jam_id = cuid2::CuidConstructor::new().with_length(6).create_id();

    let error = sqlx::query!(
        "INSERT INTO jams (id, max_song_count, host_id, name) VALUES ($1, $2, $3, $4)",
        &jam_id,
        &max_song_count,
        host_id,
        name
    )
    .execute(pool)
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

    tokio::spawn(occasional_notify(pool.clone(), jam_id.clone()));

    Ok(jam_id)
}

pub async fn occasional_notify(pool: sqlx::PgPool, jam_id: String) -> Result<(), Error> {
    use std::time::Duration;
    loop {
        log!("Occasional notify");
        if let Err(e) = notify(real_time::Changed::all(), vec![], &jam_id, &pool).await {
            eprintln!("Error notifying all, in occasional notify: {:?}", e);
        };
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
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
        pub user_id: String,
        pub name: String,
        pub album: String,
        pub duration: i32,
        pub artists: Option<Vec<String>>,
        pub image_url: String,
    }

    let song = sqlx::query_as!(
        SongDb,
        "SELECT * FROM current_songs WHERE user_id IN (SELECT id FROM users WHERE jam_id=$1)",
        jam_id
    )
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
        id: song.id,
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

pub async fn set_current_song(
    song_id: String,
    id: &IdType,
    pool: &sqlx::PgPool,
) -> Result<real_time::Changed, Error> {
    use crate::general::functions::song::get_songs;

    let mut transaction = pool.begin().await?;

    if id.is_user() {
        return Err(Error::Forbidden(
            "Only hosts can set the current song".to_string(),
        ));
    }

    let song = get_songs(pool, id)
        .await?
        .into_iter()
        .find(|song| song.id == song_id)
        .ok_or(Error::InvalidRequest(
            "Song not found in current queue".to_string(),
        ))?;

    sqlx::query!(
        "DELETE FROM current_songs WHERE user_id IN (SELECT id FROM users WHERE jam_id=$1)",
        id.jam_id()
    )
    .execute(&mut *transaction)
    .await?;

    let user_id = sqlx::query!("SELECT user_id FROM songs WHERE id=$1", song_id)
        .fetch_one(&mut *transaction)
        .await?
        .user_id;
    sqlx::query!(
        "INSERT INTO current_songs (id, user_id, name, album, duration, artists, image_url) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        song.id,
        user_id,
        song.name,
        song.album,
        song.duration as i32,
        &song.artists,
        song.image_url
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(real_time::Changed::new().current_song())
}
