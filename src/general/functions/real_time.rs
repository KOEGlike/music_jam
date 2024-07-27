use crate::general::types::*;
use strum::IntoEnumIterator;


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

pub async fn notify_all(jam_id: &str, pool: &sqlx::PgPool) -> Result<(), Error> {
    use std::sync::Arc;
    
    let jam_id = Arc::new(jam_id.to_string());
    let pool = Arc::new(pool.clone());
    let mut futures = Vec::new();
    for channel in real_time::Channels::iter() {
        let jam_id = Arc::clone(&jam_id);
        let pool = Arc::clone(&pool);
        futures.push(tokio::spawn(async move{
            notify(channel, &jam_id, &pool).await
        }));
    }
    
    let mut errors = Vec::new();
    for future in futures {
        match future.await {
            Ok(fut) => {
                if let Err(e) = fut {
                    errors.push(e.to_string())
                }
            },
            Err(e) => errors.push(e.to_string()),
        };
    }

    if !errors.is_empty() {
        return Err(Error::Database(errors.join("\n")));
    }

    Ok(())
}

pub async fn reset_votes(jam_id: &str, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("DELETE FROM votes WHERE song_id IN (SELECT id FROM songs WHERE user_id IN (SELECT id FROM users WHERE jam_id=$1));", jam_id)
        .execute(pool)
        .await?;

    notify(real_time::Channels::Votes, jam_id, pool).await?;
    Ok(())
}
