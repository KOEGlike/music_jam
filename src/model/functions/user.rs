use sqlx::Transaction;

use crate::model::types::*;

///only the jam is is used from the id
pub async fn get_users<'e>(
    executor: impl sqlx::PgExecutor<'e>,
    id: &Id,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE jam_id=$1", id.jam_id())
        .fetch_all(executor)
        .await
        .map(|users| {
            users
                .into_iter()
                .filter(|user| user.id.trim() != id.jam_id())
                .collect()
        })
}

pub async fn check_id_type<'e>(
    id: &str,
    transaction: &mut Transaction<'e, sqlx::Postgres>,
) -> Result<Id, Error> {
    // Check if the ID exists in the hosts table
    let host_check = sqlx::query!("SELECT EXISTS(SELECT 1 FROM hosts WHERE id = $1)", id)
        .fetch_one(&mut **transaction)
        .await?;

    if host_check.exists.unwrap_or(false) {
        let jam_id = sqlx::query!("SELECT id FROM jams WHERE host_id = $1", id)
            .fetch_one(&mut **transaction)
            .await?
            .id;
        return Ok(Id {
            id: IdType::Host(id.to_string()),
            jam_id,
        });
    }

    let user_check = sqlx::query!("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)", id)
        .fetch_one(&mut **transaction)
        .await?;

    if user_check.exists.unwrap_or(false) {
        let jam_id = sqlx::query!("SELECT jam_id FROM users WHERE id = $1", id)
            .fetch_one(&mut **transaction)
            .await?
            .jam_id;
        return Ok(Id {
            id: IdType::User(id.to_string()),
            jam_id,
        });
    }

    Err(Error::DoesNotExist(format!(
        "user/host with id: {}, doesn't exist",
        id
    )))
}

pub async fn kick_user<'e>(
    user_id: &str,
    executor: impl sqlx::PgExecutor<'e>,
) -> Result<real_time::Changed, Error> {
    let res = sqlx::query!("DELETE FROM users WHERE id=$1;", user_id)
        .execute(executor)
        .await?;

    if res.rows_affected() == 0 {
        return Err(Error::DoesNotExist(format!(
            "user with id: {} does not exist, could not kick",
            user_id
        )));
    }

    Ok(real_time::Changed::new().users())
}

///returns id of the created user
pub async fn create_user<'e>(
    jam_id: &str,
    image_url: &str,
    name: &str,
    executor: impl sqlx::PgExecutor<'e>,
    root: &str,
) -> Result<(String, real_time::Changed), Error> {
    use data_url::DataUrl;

    if name.is_empty() {
        return Err(Error::InvalidRequest("name is empty".to_string()));
    }

    let data_url = match DataUrl::process(image_url) {
        Ok(data_url) => data_url,
        Err(_) => return Err(Error::Decode("invalid data url".to_string())),
    };
    let bytes = match data_url.decode_to_vec() {
        Ok(bytes) => bytes.0,
        Err(_) => return Err(Error::Decode("could not decode data url".to_string())),
    };
    if data_url.mime_type().type_ != "image" {
        return Err(Error::Decode("not an image".to_string()));
    }
    let image_format = match data_url.mime_type().subtype.as_str() {
        "jpeg" | "jpg" => image::ImageFormat::Jpeg,
        "png" => image::ImageFormat::Png,
        "gif" => image::ImageFormat::Gif,
        "webp" => image::ImageFormat::WebP,
        _ => return Err(Error::Decode("unsupported image format".to_string())),
    };
    let image = match image::load_from_memory_with_format(&bytes, image_format) {
        Ok(image) => image,
        Err(e) => {
            return Err(Error::Decode(format!(
                "could not decode image, error: {}",
                e
            )))
        }
    };
    let image = image
        .resize(256, 256, image::imageops::FilterType::Lanczos3)
        .crop_imm(0, 0, 256, 256);

    let user_id = cuid2::create_id();
    let image_path = format!("{}/uploads/{}.webp", root, user_id);

    match image.save(image_path) {
        Ok(_) => (),
        Err(e) => {
            return Err(Error::FileSystem(format!(
                "could not save image, error: {}",
                e
            )))
        }
    };

    sqlx::query!(
        "INSERT INTO users(id, jam_id, name) VALUES ($1, $2, $3)",
        user_id,
        jam_id.to_lowercase(),
        name,
    )
    .execute(executor)
    .await?;

    Ok((user_id, real_time::Changed::new().users()))
}
