use crate::general::functions::{get_access_token, notify};
use crate::general::types::*;
use leptos::logging::*;
use rspotify::model::Image;
use rspotify::model::TrackId;
use std::collections::HashMap;

pub async fn remove_song(song_id: &str, id: &IdType, pool: &sqlx::PgPool) -> Result<(), Error> {
    if let IdType::User(id) = id {
        let song_user_id = sqlx::query!(
            "SELECT * FROM songs WHERE id=$1 AND user_id=$2",
            song_id,
            id.id
        )
        .fetch_optional(pool)
        .await?;
        if song_user_id.is_none() {
            return Err(Error::Forbidden(
                "this song was not added by the user who wants to remove it".to_string(),
            ));
        }
    }

    sqlx::query!("DELETE FROM songs WHERE id=$1;", song_id)
        .execute(pool)
        .await?;
    notify(real_time::Channels::Songs, id.jam_id(), pool).await?;
    Ok(())
}

pub async fn get_songs(pool: &sqlx::PgPool, id: &IdType) -> Result<Vec<Song>, sqlx::Error> {
    struct SongDb {
        pub id: String,
        pub user_id: String,
        pub name: String,
        pub album: String,
        pub duration: i32,
        pub votes: Option<i64>,
        pub artists: Option<Vec<String>>,
        pub image_url: String,
    }

    let vec = sqlx::query_as!(
        SongDb,
        "SELECT s.id, s.artists, s.image_url, s.user_id, s.name, s.album, s.duration, COUNT(v.id) AS votes
        FROM songs s
        JOIN users u ON s.user_id = u.id
        LEFT JOIN votes v ON s.id = v.song_id
        WHERE u.jam_id = $1
        GROUP BY s.id
        ORDER BY votes DESC, s.id DESC;",
        &id.jam_id()
    )
    .fetch_all(pool)
    .await?;

    let votes: HashMap<String, Vote> = match id {
        IdType::Host(_) => vec
            .iter()
            .map(|song| {
                (
                    song.id.clone(),
                    Vote {
                        votes: song.votes.unwrap_or(0) as u64,
                        have_you_voted: None,
                    },
                )
            })
            .collect(),
        IdType::User(id) => {
            let votes = sqlx::query!("SELECT song_id FROM votes WHERE user_id=$1;", id.id)
                .fetch_all(pool)
                .await?
                .into_iter()
                .map(|vote| vote.song_id)
                .collect::<Vec<String>>();
            vec.iter()
                .map(|song| {
                    (
                        song.id.clone(),
                        Vote {
                            votes: song.votes.unwrap_or(0) as u64,
                            have_you_voted: Some(votes.contains(&song.id)),
                        },
                    )
                })
                .collect()
        }
    };

    let songs = vec
        .into_iter()
        .map(|song| Song {
            votes: votes.get(&song.id).cloned().unwrap_or(Vote {
                votes: 0,
                have_you_voted: match id {
                    IdType::Host(_) => None,
                    IdType::User(_) => Some(false),
                },
            }),
            id: song.id,
            user_id: {
                if let IdType::User(ref id) = id {
                    if id.id == song.user_id {
                        Some(song.user_id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            name: song.name,
            artists: song
                .artists
                .unwrap_or(vec!["no artist found in cache, this is a bug".to_string()]),
            album: song.album,
            duration: song.duration as u16,
            image_url: song.image_url,
        })
        .collect::<Vec<_>>();

    Ok(songs)
}

pub async fn add_song(
    song_id: &str,
    user_id: &str,
    jam_id: &str,
    pool: &sqlx::PgPool,
    credentials: SpotifyCredentials,
) -> Result<(), Error> {
    use rspotify::prelude::*;
    use rspotify::AuthCodeSpotify;
    log!("adding song, with id: {}", song_id);

    let token = get_access_token(pool, jam_id, credentials).await?;
    let client = AuthCodeSpotify::from_token(token);
    let track_id = TrackId::from_id(song_id)?;
    let song = client.track(track_id, None).await?;

    sqlx::query!(
        "INSERT INTO songs 
            (id, user_id, name, album, duration, image_url, artists) 
        VALUES 
            ($1, $2, $3, $4, $5, $6, $7);",
        song_id,
        user_id,
        song.name,
        song.album.name,
        song.duration.num_milliseconds() as i32,
        song.album.images[0].url,
        &song
            .artists
            .into_iter()
            .map(|a| a.name)
            .collect::<Vec<String>>()
    )
    .execute(pool)
    .await?;

    notify(real_time::Channels::Songs, jam_id, pool).await?;
    Ok(())
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
        user_id: None,
        name: song.name,
        artists: song
            .artists
            .unwrap_or(vec!["no artist found in cache, this is a bug".to_string()]),
        album: song.album,
        duration: song.duration as u16,
        image_url: song.image_url,
    }))
}
