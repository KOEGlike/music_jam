use crate::model::functions::get_access_token;
use crate::model::types::*;
use itertools::Itertools;
use leptos::logging::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rspotify::model::TrackId;
use std::collections::HashMap;

pub async fn remove_song(
    song_id: &str,
    id: &Id,
    pool: &sqlx::PgPool,
) -> Result<real_time::Changed, Error> {
    if let IdType::User(id) = &id.id {
        let song_user_id = sqlx::query!(
            "SELECT * FROM songs WHERE id=$1 AND user_id=$2",
            song_id,
            id
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
    Ok(real_time::Changed::new().songs())
}

pub async fn get_top_song(pool: &sqlx::PgPool, jam_id: String) -> Result<Option<Song>, Error> {
    let id = Id {
        id: IdType::General,
        jam_id,
    };

    let songs = get_songs(pool, &id).await?;
    if songs.is_empty() {
        return Ok(None);
    }

    let mut songs = songs.into_iter().max_set_by_key(|s| s.votes.votes);
    songs.shuffle(&mut thread_rng());
    Ok(songs.into_iter().next())
}

pub async fn get_songs(pool: &sqlx::PgPool, id: &Id) -> Result<Vec<Song>, sqlx::Error> {
    struct SongDb {
        pub id: String,
        pub spotify_id: String,
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
        "SELECT s.id, s.spotify_id ,s.artists, s.image_url, s.user_id, s.name, s.album, s.duration, COUNT(v.id) AS votes
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

    let votes: HashMap<String, Vote> = match &id.id {
        IdType::Host(_) | IdType::General => vec
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
            let votes = sqlx::query!("SELECT song_id FROM votes WHERE user_id=$1;", id)
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
        .filter(|song| song.user_id.trim() != id.jam_id())
        .map(|song| Song {
            votes: votes.get(&song.id).cloned().unwrap_or(Vote {
                votes: 0,
                have_you_voted: match id.id {
                    IdType::Host(_) | IdType::General => None,
                    IdType::User(_) => Some(false),
                },
            }),
            id: Some(song.id),
            spotify_id: song.spotify_id,
            user_id: {
                match &id.id {
                    IdType::User(id) => {
                        if id == &song.user_id {
                            Some(song.user_id)
                        } else {
                            None
                        }
                    }
                    IdType::General => Some(song.user_id),
                    IdType::Host(_) => None,
                }
            },
            name: song.name,
            artists: song
                .artists
                .unwrap_or(vec!["no artist found in cache, this is a bug".to_string()]),
            album: song.album,
            duration: song.duration as u32,
            image_url: song.image_url,
        })
        .collect::<Vec<_>>();

    Ok(songs)
}

pub async fn add_song(
    spotify_song_id: &str,
    user_id: &str,
    jam_id: &str,
    pool: &sqlx::PgPool,
    credentials: SpotifyCredentials,
) -> Result<real_time::Changed, Error> {
    use rspotify::prelude::*;
    use rspotify::AuthCodeSpotify;
    println!("adding song, with id: {}", spotify_song_id);

    let mut transaction = pool.begin().await?;

    let does_song_exist = sqlx::query!("SELECT EXISTS(SELECT 1 FROM songs WHERE spotify_id=$1 AND user_id IN (SELECT id FROM users WHERE jam_id=$2))", spotify_song_id, jam_id)
        .fetch_one(&mut *transaction)
        .await?;

    if does_song_exist.exists.unwrap_or(false) {
        return Err(Error::SongAlreadyInJam);
    }

    let amount_of_songs = sqlx::query!("SELECT COUNT(*) FROM songs WHERE user_id=$1", user_id)
        .fetch_one(&mut *transaction)
        .await?
        .count
        .unwrap_or(0);

    let max_amount_of_songs = sqlx::query!("SELECT max_song_count FROM jams WHERE id=$1", jam_id)
        .fetch_one(&mut *transaction)
        .await?
        .max_song_count;

    if amount_of_songs as i16 >= max_amount_of_songs {
        return Err(Error::UserHasTooTheMaxSongAmount);
    }

    let token = get_access_token(pool, jam_id, credentials).await?;
    let client = AuthCodeSpotify::from_token(token);
    let track_id = TrackId::from_id(spotify_song_id)?;
    let song = client.track(track_id, None).await?;

    sqlx::query!(
        "INSERT INTO songs 
            (id, user_id, name, album, duration, image_url, artists, spotify_id) 
        VALUES 
            ($1, $2, $3, $4, $5, $6, $7, $8);",
        cuid2::create_id(),
        user_id,
        song.name,
        song.album.name,
        song.duration.num_milliseconds() as i32,
        song.album.images[0].url,
        &song
            .artists
            .into_iter()
            .map(|a| a.name)
            .collect::<Vec<String>>(),
        spotify_song_id,
    )
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;

    Ok(real_time::Changed::new().songs())
}
