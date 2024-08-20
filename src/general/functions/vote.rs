
use crate::general::types::*;


pub async fn add_vote(song_id: &str, user_id: &Id, pool: &sqlx::PgPool) -> Result<real_time::Changed, Error> {
    let vote_exists = sqlx::query!(
        "SELECT * FROM votes WHERE song_id=$1 AND user_id=$2;",
        song_id,
        user_id.id
    )
    .fetch_optional(pool)
    .await?;

    if vote_exists.is_some() {
        return Err(Error::Forbidden(
            "user has already voted for this song".to_string(),
        ));
    }

    sqlx::query!(
        "INSERT INTO votes (song_id, user_id, id) VALUES ($1, $2, $3);",
        song_id,
        user_id.id,
        format!("{}{}", song_id, user_id.id)
    )
    .execute(pool)
    .await?;

    Ok(real_time::Changed::new().votes())
}

pub async fn remove_vote(song_id: &str, user_id: &Id, pool: &sqlx::PgPool) -> Result<real_time::Changed, Error> {
    let vote_exists = sqlx::query!(
        "SELECT * FROM votes WHERE song_id=$1 AND user_id=$2;",
        song_id,
        user_id.id
    )
    .fetch_optional(pool)
    .await?;

    if vote_exists.is_none() {
        return Err(Error::Forbidden(
            "user has not voted for this song".to_string(),
        ));
    }

    sqlx::query!(
        "DELETE FROM votes WHERE song_id=$1 AND user_id=$2;",
        song_id,
        user_id.id
    )
    .execute(pool)
    .await?;

    Ok(real_time::Changed::new().votes())
}

pub async fn get_votes(pool: &sqlx::PgPool, id: &IdType) -> Result<Votes, sqlx::Error> {
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
        id.jam_id()
    )
    .fetch_all(pool)
    .await?;
    let votes = match id {
        IdType::Host(_) => vec
            .into_iter()
            .map(|v| {
                (
                    v.song_id,
                    Vote {
                        votes: v.votes_nr.unwrap_or(0) as u64,
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
            vec.into_iter()
                .map(|v| {
                    let contains = votes.contains(&v.song_id);
                    (
                        v.song_id,
                        Vote {
                            votes: v.votes_nr.unwrap_or(0) as u64,
                            have_you_voted: Some(contains),
                        },
                    )
                })
                .collect()
        }
    };

    Ok(votes)
}

pub async fn reset_votes(jam_id: &str, pool: &sqlx::PgPool) -> Result<real_time::Changed, sqlx::Error> {
    sqlx::query!("DELETE FROM votes WHERE song_id IN (SELECT id FROM songs WHERE user_id IN (SELECT id FROM users WHERE jam_id=$1));", jam_id)
        .execute(pool)
        .await?;
    
    Ok(real_time::Changed::new().votes())
}