use crate::app::general::*;
use sqlx::Postgres;

pub async fn notify(
    channel: real_time::Channels,
    jam_id: &str,
    pool: &sqlx::Pool<Postgres>,
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
    host_id: &str,
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
        "SELECT * FROM access_tokens WHERE id=(SELECT access_token FROM hosts WHERE id=$1)",
        host_id
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
    sqlx::query_as!(
        Song,
        "SELECT s.*, COUNT(v.id) AS votes
        FROM songs s
        JOIN users u ON s.user_id = u.id
        LEFT JOIN votes v ON s.id = v.song_id
        WHERE u.jam_id = $1
        GROUP BY s.id
        ORDER BY votes DESC, s.id DESC;",
        jam_id
    )
    .fetch_all(pool)
    .await
}

pub async fn get_votes(pool: &sqlx::PgPool, jam_id: &str) -> Result<Votes, sqlx::Error> {
    struct VotesDb {
        pub song_id: String,
        pub votes_nr: Option<i64>,
    }
    let vec=sqlx::query_as!(
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
    let map=vec.into_iter().map(|v|(v.song_id,v.votes_nr.unwrap_or(0))).collect();
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
    pool: &sqlx::Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO songs (id, user_id) VALUES ($1, $2)",
        song_id,
        user_id,
    )
    .execute(pool)
    .await?;

    notify(real_time::Channels::Songs, jam_id, pool).await?;
    Ok(())
}

pub async fn remove_song(
    song_id: &str,
    jam_id: &str,
    pool: &sqlx::Pool<Postgres>,
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
    pool: &sqlx::Pool<Postgres>,
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
    pool: &sqlx::Pool<Postgres>,
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
    pool: &sqlx::Pool<Postgres>,
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
