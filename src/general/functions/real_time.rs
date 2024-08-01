use crate::general::types::*;

/// only the jam id is used form the id
/// some fields such as songs and votes have different outputs depending on the id type
pub async fn notify(
    changed: real_time::Changed,
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    let update = real_time::Update::from_changed_non_specific(changed, jam_id, pool).await;
    let update=real_time::ChannelUpdate{ update, changed };
    let update = serde_json::to_string(&update).unwrap();

    sqlx::query!("SELECT pg_notify($1,$2)", jam_id, update)
        .execute(pool)
        .await?;
    Ok(())
}
