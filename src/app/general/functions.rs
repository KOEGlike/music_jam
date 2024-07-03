use crate::app::general::types::*;
use rspotify::model::{Image, SearchResult, TrackId};

pub async fn notify(
    channel: real_time::Channels,
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    let channel: String = channel.into();
    let channel = format!("{}_{}", jam_id, channel);
    sqlx::query!("SELECT pg_notify($1,'notified')", channel)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_access_token(
    pool: &sqlx::PgPool,
    jam_id: &str,
) -> Result<rspotify::Token, sqlx::Error> {
    #[allow(dead_code)]
    struct AccessTokenDb {
        pub refresh_token: String,
        pub access_token: String,
        pub expires_at: i64,
        pub scope: String,
        pub id: String,
    }

    let token = sqlx::query_as!(
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

    Ok(rspotify::Token {
        access_token: token.access_token,
        expires_in,
        expires_at,
        refresh_token: Some(token.refresh_token),
        scopes: rspotify::scopes!(token.scope),
    })
}

pub async fn get_songs(pool: &sqlx::PgPool, jam_id: &str) -> Result<Vec<Song>, sqlx::Error> {
    struct SongDb {
        pub id: String,
        pub user_id: String,
        pub name: String,
        pub album: String,
        pub duration: i32,
        pub image_id: String,
        pub votes: Option<i64>,
        pub artists: Option<Vec<String>>,
        pub image_width: Option<i64>,
        pub image_height: Option<i64>,
        pub image_url: String,
    }

    let vec = sqlx::query_as!(
        SongDb,
        "SELECT s.id, s.user_id, s.name, s.album, s.duration, s.image_id, i.url AS image_url, i.width AS image_width, i.height AS image_height, COUNT(v.id) AS votes, ARRAY_AGG(a.name) AS artists
        FROM songs s
        JOIN users u ON s.user_id = u.id
        LEFT JOIN votes v ON s.id = v.song_id
        LEFT JOIN artists a ON s.id = a.song_id
        LEFT JOIN images i ON s.image_id = i.id
        WHERE u.jam_id = $1
        GROUP BY s.id, i.id
        ORDER BY votes DESC, s.id DESC;",
        jam_id
    )
    .fetch_all(pool)
    .await?;



    let songs = vec
        .into_iter()
        .map(|song| Song {
            id: song.id,
            user_id: Some(song.user_id),
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
            votes: song.votes.unwrap_or(0),
        })
        .collect::<Vec<_>>();

    Ok(songs)
}

pub async fn get_votes(pool: &sqlx::PgPool, jam_id: &str) -> Result<Votes, sqlx::Error> {
    struct VotesDb {
        pub song_id: String,
        pub votes_nr: Option<i64>,
    }
    let vec = sqlx::query_as!(
        VotesDb,
        "SELECT s.id AS song_id, COUNT(v.id) AS votes_nr
        FROM songs s
        JOIN users u ON s.user_id = u.id
        LEFT JOIN votes v ON s.id = v.song_id
        WHERE u.jam_id = $1
        GROUP BY s.id
        ORDER BY votes_nr DESC",
        jam_id
    )
    .fetch_all(pool)
    .await?;
    let map = vec
        .into_iter()
        .map(|v| (v.song_id, v.votes_nr.unwrap_or(0)))
        .collect();
    Ok(map)
}

pub async fn get_users(pool: &sqlx::PgPool, jam_id: &str) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE jam_id=$1", jam_id)
        .fetch_all(pool)
        .await
}

pub async fn add_song(
    song_id: &str,
    user_id: &str,
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<(), real_time::Error> {
    use rspotify::prelude::*;
    use rspotify::AuthCodeSpotify;

    let token = get_access_token(pool, jam_id).await?;
    let client = AuthCodeSpotify::from_token(token);
    let track_id = TrackId::from_id(song_id)?;
    let song = client.track(track_id, None).await?;

    let mut transaction = pool.begin().await?;
    let image_id = cuid2::create_id();

    if let Some(width) = song.album.images[0].width {
        if let Some(height) = song.album.images[0].height {
            sqlx::query!(
                "INSERT INTO images (id, url, width, height) VALUES ($1, $2, $3, $4);",
                &image_id,
                &song.album.images[0].url,
                width as i32,
                height as i32
            )
            .execute(&mut *transaction)
            .await?;
        }
    } else {
        sqlx::query!(
            "INSERT INTO images (id, url) VALUES ($1, $2);",
            &image_id,
            &song.album.images[0].url
        )
        .execute(&mut *transaction)
        .await?;
    }

    sqlx::query!("INSERT INTO songs (id, user_id, name, album, duration, image_id) VALUES ($1, $2, $3, $4, $5, $6);",
        song_id,
        user_id,
        song.name,
        song.album.name,
        song.duration.num_seconds() as i32,
        &image_id
    ).execute( &mut *transaction).await?;

    for artist in song.artists {
        sqlx::query!(
            "INSERT INTO artists (song_id, name) VALUES ($1, $2);",
            song_id,
            artist.name
        )
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;

    notify(real_time::Channels::Songs, jam_id, pool).await?;
    Ok(())
}

pub async fn search(
    query: &str,
    pool: &sqlx::PgPool,
    jam_id: &str,
) -> Result<Vec<Song>, real_time::Error> {
    use rspotify::prelude::*;
    use rspotify::AuthCodeSpotify;

    let token = get_access_token(pool, jam_id).await?;
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
        return Err(real_time::Error::Spotify("Error in search".to_string()));
    };

    let songs = songs
        .items
        .iter()
        .map(|track| {
            let id = match track.id.clone() {
                Some(id) => id.to_string(),
                None => "lol this is not a local song, this is a bug".to_string(),
            };

            Song {
                id,
                user_id: None,
                name: track.name.clone(),
                artists: track.artists.iter().map(|a| a.name.clone()).collect(),
                album: track.album.name.clone(),
                duration: track.duration.num_seconds() as u16,
                image: track.album.images[0].clone(),
                votes: 0,
            }
        })
        .collect::<Vec<Song>>();

    Ok(songs)
}

pub async fn remove_song(
    song_id: &str,
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM songs WHERE id=$1;", song_id)
        .execute(pool)
        .await?;

    notify(real_time::Channels::Songs, jam_id, pool).await?;
    Ok(())
}

pub async fn add_vote(
    song_id: &str,
    user_id: &str,
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO votes (song_id, user_id) VALUES ($1, $2);",
        song_id,
        user_id,
    )
    .execute(pool)
    .await?;

    notify(real_time::Channels::Votes, jam_id, pool).await?;
    Ok(())
}

pub async fn remove_vote(
    song_id: &str,
    user_id: &str,
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM votes WHERE song_id=$1 AND user_id=$2;",
        song_id,
        user_id,
    )
    .execute(pool)
    .await?;

    notify(real_time::Channels::Votes, jam_id, pool).await?;
    Ok(())
}

pub async fn kick_user(
    user_id: &str,
    host_id: &str,
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    //check if the jam that the user is in is owned by the host
    struct JamId {
        id: String,
    }

    let jam_id = sqlx::query_as!(JamId, "SELECT id FROM jams WHERE host_id=$1;", host_id)
        .fetch_one(pool)
        .await?;
    let jam_id = jam_id.id;

    sqlx::query!(
        "DELETE FROM users WHERE id=$1 AND jam_id=$2; ",
        user_id,
        jam_id
    )
    .execute(pool)
    .await?;

    notify(real_time::Channels::Users, &jam_id, pool).await?;
    Ok(())
}
