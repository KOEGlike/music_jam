use crate::general::functions::{notify, get_access_token};
use crate::general::types::*;
use rspotify::model::Image;
use std::collections::HashMap;
use leptos::logging::*;
use rspotify::model::TrackId;

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
        pub image_width: Option<i64>,
        pub image_height: Option<i64>,
        pub image_url: String,
    }
    let vec = sqlx::query_as!(
        SongDb,
        "SELECT s.id, s.user_id, s.name, s.album, s.duration, i.url AS image_url, i.width AS image_width, i.height AS image_height, COUNT(v.id) AS votes, ARRAY_AGG(a.name) AS artists
        FROM songs s
        JOIN users u ON s.user_id = u.id
        LEFT JOIN votes v ON s.id = v.song_id
        LEFT JOIN artists a ON s.id = a.song_id
        LEFT JOIN images i ON i.song_id = s.id
        WHERE u.jam_id = $1
        GROUP BY s.id, i.id
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
            image: Image {
                url: song.image_url,
                width: song.image_width.map(|width| width as u32),
                height: song.image_height.map(|height| height as u32),
            },
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

    let mut transaction = pool.begin().await?;
    let image_id = cuid2::create_id();

    sqlx::query!(
        "INSERT INTO songs (id, user_id, name, album, duration) VALUES ($1, $2, $3, $4, $5);",
        song_id,
        user_id,
        song.name,
        song.album.name,
        song.duration.num_milliseconds() as i32,
    )
    .execute(&mut *transaction)
    .await?;

    if let Some(width) = song.album.images[0].width {
        if let Some(height) = song.album.images[0].height {
            sqlx::query!(
                "INSERT INTO images (id, url, width, height, song_id) VALUES ($1, $2, $3, $4, $5);",
                &image_id,
                &song.album.images[0].url,
                width as i32,
                height as i32,
                &song_id
            )
            .execute(&mut *transaction)
            .await?;
        }
    } else {
        sqlx::query!(
            "INSERT INTO images (id, url, song_id) VALUES ($1, $2, $3);",
            &image_id,
            &song.album.images[0].url,
            &song_id
        )
        .execute(&mut *transaction)
        .await?;
    }

    for artist in song.artists {
        sqlx::query!(
            "INSERT INTO artists (song_id, name, id) VALUES ($1, $2, $3);",
            song_id,
            artist.name,
            artist.id.unwrap().id().to_owned()
        )
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    notify(real_time::Channels::Songs, jam_id, pool).await?;
    Ok(())
}

