use crate::model::types::*;

/// only the jam id is used form the id
/// some fields such as songs and votes have different outputs depending on the id type
pub async fn notify(
    changed: real_time::Changed,
    errors: Vec<Error>,
    jam_id: &str,
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {
    if changed.has_changed() {
        let update = real_time::ChannelUpdate { errors, changed };
        let update = serde_json::to_string(&update).unwrap();

        sqlx::query!("SELECT pg_notify($1,$2)", jam_id, update)
            .execute(pool)
            .await?;
    }
    Ok(())
}
