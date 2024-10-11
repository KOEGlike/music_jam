use std::collections::{HashMap, HashSet};

use crate::model::{get_current_song, types::*};

pub async fn add_vote<'e>(
    song_id: &str,
    user_id: &str,
    executor: impl sqlx::PgExecutor<'e>,
) -> Result<real_time::Changed, Error> {
    let result = sqlx::query!(
        "INSERT INTO votes (song_id, user_id, id)
         VALUES ($1, $2, $3)
         ON CONFLICT (song_id, user_id) DO NOTHING",
        song_id,
        user_id,
        format!("{}{}", song_id, user_id)
    )
    .execute(executor)
    .await?;

    if result.rows_affected() == 0 {
        eprintln!("vote exists, returning error");
        return Err(Error::Forbidden(
            "user has already voted for this song".to_string(),
        ));
    }

    Ok(real_time::Changed::new().votes())
}

pub async fn remove_vote<'e>(
    song_id: &str,
    user_id: &str,
    executor: impl sqlx::PgExecutor<'e>,
) -> Result<real_time::Changed, Error> {
    let result = sqlx::query!(
        "DELETE FROM votes WHERE song_id=$1 AND user_id=$2;",
        song_id,
        user_id
    )
    .execute(executor)
    .await?;

    if result.rows_affected() == 0 {
        eprintln!("vote does not exist, returning error");
        return Err(Error::Forbidden(
            "user has not voted for this song".to_string(),
        ));
    }

    Ok(real_time::Changed::new().votes())
}

pub async fn get_votes<'e>(
    transaction: &mut sqlx::Transaction<'e, sqlx::Postgres>,
    id: &Id,
) -> Result<Votes, sqlx::Error> {
    struct VotesDb {
        pub song_id: String,
        pub votes_nr: Option<i64>,
    }

    // Fetch the vote counts for all songs in the current jam
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
    .fetch_all(&mut **transaction)
    .await?;

    // Handle vote data based on the type of ID
    let mut votes: HashMap<String, Vote> = match &id.id {
        IdType::Host(_) | IdType::General => vec
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
        IdType::User(user_id) => {
            // Fetch the songs the user has voted for
            let user_votes = sqlx::query!("SELECT song_id FROM votes WHERE user_id = $1;", user_id)
                .fetch_all(&mut **transaction)
                .await?
                .into_iter()
                .map(|vote| vote.song_id)
                .collect::<HashSet<String>>(); // Collect into a HashSet for efficient lookup

            vec.into_iter()
                .map(|v| {
                    let user_voted = user_votes.contains(&v.song_id);
                    (
                        v.song_id,
                        Vote {
                            votes: v.votes_nr.unwrap_or(0) as u64,
                            have_you_voted: Some(user_voted),
                        },
                    )
                })
                .collect()
        }
    };

    // Optionally remove the current song from the vote list
    if let Some(current_song) = get_current_song(id.jam_id(), &mut **transaction).await? {
        votes.remove_entry(current_song.id.unwrap().as_str());
    }

    Ok(votes)
}

pub async fn reset_votes<'e>(
    jam_id: &str,
    executor: impl sqlx::PgExecutor<'e>,
) -> Result<real_time::Changed, sqlx::Error> {
    sqlx::query!("DELETE FROM votes WHERE song_id IN (SELECT id FROM songs WHERE user_id IN (SELECT id FROM users WHERE jam_id=$1));", jam_id)
        .execute(executor)
        .await?;

    Ok(real_time::Changed::new().votes())
}
