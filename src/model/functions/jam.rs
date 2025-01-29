use super::notify;
use crate::model::types::*;
use real_time::Changed;

pub async fn get_jam<'e>(jam_id: &str, executor: impl sqlx::PgExecutor<'e>) -> Result<Jam, Error> {
    let jam = match sqlx::query!("SELECT * FROM jams WHERE id = $1", jam_id.to_lowercase())
        .fetch_one(executor)
        .await
    {
        Ok(jam) => jam,
        Err(sqlx::Error::RowNotFound) => {
            return Err(Error::DoesNotExist(format!(
                "jam with id {} does not exist, could not get jam",
                jam_id
            )));
        }
        Err(e) => {
            return Err(e.into());
        }
    };
    Ok(Jam {
        id: jam.id,
        name: jam.name,
        max_song_count: jam.max_song_count as u8,
    })
}

pub async fn create_host<'e>(
    code: String,
    host_id: String,
    spotify_credentials: &SpotifyCredentials,
    reqwest_client: &reqwest::Client,
    executor: impl sqlx::PgExecutor<'e>,
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
                .execute(executor)
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
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn delete_jam<'e>(
    jam_id: &str,
    executor: impl sqlx::PgExecutor<'e>,
) -> Result<real_time::Changed, Error> {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let res = sqlx::query!("DELETE FROM jams WHERE id = $1", jam_id)
        .execute(executor)
        .await?;
    if res.rows_affected() == 0 {
        return Err(Error::DoesNotExist(format!(
            "jam with id {} does not exist, could not delete jam, maybe it was already deleted",
            jam_id
        )));
    }
    Ok(real_time::Changed::new().ended())
}

pub async fn create_jam<'e>(
    name: &str,
    host_id: &str,
    max_song_count: i16,
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    credentials: SpotifyCredentials,
) -> Result<JamId, Error> {
    println!("checking if jam exists");
    let jam_exists = sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM jams WHERE host_id=$1)",
        host_id
    )
    .fetch_one(&mut **transaction)
    .await?
    .exists
    .unwrap_or(false);

    if jam_exists {
        println!("host already in jam");
        match sqlx::query!("SELECT id FROM jams WHERE host_id=$1", host_id)
            .fetch_one(&mut **transaction)
            .await
        {
            Ok(jam) => {
                return Err(Error::HostAlreadyInJam { jam_id: jam.id });
            }
            Err(sqlx::Error::RowNotFound) => {
                println!("idk what happened");
                return Err(Error::DoesNotExist(format!(
                    "jam with host id {} does not exist, oooooo i don't know what happened",
                    host_id
                )));
            }
            Err(e) => {
                println!("error getting already existing jam id: {:#?}", e);
                return Err(e.into());
            }
        };
    }

    let jam_id = cuid2::CuidConstructor::new()
        .with_length(6)
        .create_id()
        .to_lowercase();

    println!("trying to insert jam");

    sqlx::query!(
        "INSERT INTO jams (id, max_song_count, host_id, name) VALUES ($1, $2, $3, $4)",
        &jam_id,
        &max_song_count,
        host_id,
        name
    )
    .execute(&mut **transaction)
    .await?;

    println!("inserted new jam");

    println!("trying to insert jam user");

    sqlx::query!(
        "INSERT INTO users (id, jam_id, name) VALUES ($1, $1, $2)",
        jam_id,
        name
    )
    .execute(&mut **transaction)
    .await?;

    println!("getting next song");

    let song = get_next_song(&jam_id, &mut *transaction, credentials).await?;
    println!("trying to set current song");
    let changed = set_current_song(&song, &jam_id, &mut *transaction).await?;
    println!("trying to notify");
    notify(changed, vec![], &jam_id, &mut *transaction).await?;

    println!("successfully created jam with id: {}", jam_id);
    Ok(jam_id)
}

pub async fn set_current_song_position(
    jam_id: &str,
    percentage: f32,
    credentials: SpotifyCredentials,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<real_time::Changed, Error> {
    if !(0.0..=1.0).contains(&percentage) {
        return Err(Error::InvalidRequest(
            "Percentage must be between 0 and 1".to_string(),
        ));
    }

    if percentage > 0.99 {
        let changed = go_to_next_song(jam_id, transaction, credentials).await?;
        println!("new song");
        return Ok(changed.position());
    }

    let res = sqlx::query!(
        "UPDATE jams SET song_position = $1 WHERE id = $2",
        percentage,
        jam_id
    )
    .execute(&mut **transaction)
    .await?;

    if res.rows_affected() == 0 {
        return Err(Error::DoesNotExist(format!(
            "jam with id {} does not exist, could not set song position",
            jam_id
        )));
    }

    Ok(real_time::Changed::new().position())
}

pub async fn get_current_song_position<'e>(
    jam_id: &str,
    executor: impl sqlx::PgExecutor<'e>,
) -> Result<f32, Error> {
    let row = match sqlx::query!("SELECT song_position FROM jams WHERE id = $1", jam_id)
        .fetch_one(executor)
        .await
    {
        Ok(row) => row,
        Err(sqlx::Error::RowNotFound) => {
            return Err(Error::DoesNotExist(format!(
                "jam with id {} does not exist, could not get song position",
                jam_id
            )))
        }
        Err(e) => return Err(e.into()),
    };

    Ok(row.song_position)
}

pub async fn get_current_song<'e>(
    jam_id: &str,
    executor: impl sqlx::PgExecutor<'e>,
) -> Result<Option<Song>, Error> {
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

    let song = match sqlx::query_as!(SongDb, "SELECT * FROM songs WHERE user_id=$1", jam_id)
        .fetch_optional(executor)
        .await
    {
        Ok(song) => song,
        Err(sqlx::Error::RowNotFound) => {
            return Err(Error::DoesNotExist(format!(
                "song with user id {} does not exist, could not get current song",
                jam_id
            )))
        }
        Err(e) => return Err(e.into()),
    };

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
pub async fn set_current_song<'e>(
    song: &Song,
    jam_id: &str,
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
) -> Result<real_time::Changed, Error> {
    println!("set song: {:#?}", song);
    let song_id = cuid2::create_id(); // Generate a new song ID

    // Try to update the song first
    let res = sqlx::query!(
        "UPDATE songs 
        SET name = $1, album = $2, duration = $3, artists = $4, image_url = $5, spotify_id = $6 
        WHERE user_id = $7",
        song.name,
        song.album,
        song.duration as i32,
        &song.artists,
        song.image_url,
        song.spotify_id,
        jam_id
    )
    .execute(&mut **transaction)
    .await?;

    // If no rows were affected, insert the new song
    if res.rows_affected() == 0 {
        sqlx::query!(
            "INSERT INTO songs (id, name, album, duration, artists, image_url, spotify_id, user_id) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            song_id,
            song.name,
            song.album,
            song.duration as i32,
            &song.artists,
            song.image_url,
            song.spotify_id,
            jam_id
        )
        .execute(&mut **transaction)
        .await?;
    }

    Ok(real_time::Changed::new().current_song()) // Return success
}

pub async fn dose_jam_exist<'e>(
    jam_id: &str,
    executor: impl sqlx::PgExecutor<'e>,
) -> Result<bool, Error> {
    sqlx::query!(
        "SELECT EXISTS(SELECT 1 FROM jams WHERE id=$1)",
        jam_id.to_lowercase()
    )
    .fetch_one(executor)
    .await
    .map(|b| b.exists.unwrap_or(false))
    .map_err(|e| e.into())
}

pub async fn get_next_song<'e>(
    jam_id: &str,
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    credentials: SpotifyCredentials,
) -> Result<Song, Error> {
    use super::*;
    let top_song = get_top_song(transaction, jam_id.to_string()).await?;
    if let Some(s) = top_song {
        return Ok(s);
    }

    if let Ok(Some(s)) = get_next_song_from_player(jam_id, transaction, credentials.clone()).await {
        return Ok(s);
    }

    match get_song_recommendation(credentials.clone(), jam_id, transaction).await {
        Ok(s) => return Ok(s),
        Err(e) => {
            eprintln!("error getting song recommendation: {}", e);
        }
    }

    let never_gonna = search("Never gonna give you up", transaction, jam_id, credentials)
        .await?
        .remove(0);
    Ok(never_gonna)
}

pub async fn go_to_next_song<'e>(
    jam_id: &str,
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    credentials: SpotifyCredentials,
) -> Result<Changed, Error> {
    use super::*;

    let top_song = get_next_song(jam_id, transaction, credentials.clone()).await?;

    let changed = set_current_song(&top_song, jam_id, transaction).await?;

    let changed = reset_votes(jam_id, &mut **transaction)
        .await?
        .merge_with_other(changed);

    play_song(&top_song.spotify_id, jam_id, transaction, credentials).await?;

    Ok(changed)
}
