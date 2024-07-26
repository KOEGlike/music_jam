use crate::general::types::*;

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

pub async fn notify_all(jam_id: &str, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let err = tokio::join!(
        notify(real_time::Channels::Songs, jam_id, pool),
        notify(real_time::Channels::Users, jam_id, pool),
        notify(real_time::Channels::Votes, jam_id, pool)
    );

    err.0?;
    err.1?;
    err.2?;

    Ok(())
}

pub async fn reset_votes(jam_id: &str, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM votes WHERE song_id IN (SELECT id FROM songs WHERE user_id IN (SELECT id FROM users WHERE jam_id=$1));", jam_id)
        .execute(pool)
        .await?;

    notify(real_time::Channels::Votes, jam_id, pool).await?;
    Ok(())
}
