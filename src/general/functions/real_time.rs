use crate::general::types::*;


/// only the jam id is used form the id
pub async fn notify(
    changed: real_time::Changed,
    id: &IdType,
    pool: &sqlx::PgPool,
) -> Result<(), sqlx::Error> {

    let update=real_time::Update::from_changed(changed, id, pool).await;
    let update = serde_json::to_string(&update).unwrap();
    //let update=rmp_serde::to_vec(&update).unwrap();
    sqlx::query!("SELECT pg_notify($1,$2)", id.jam_id(), update)
        .execute(pool)
        .await?;
    Ok(())
}
