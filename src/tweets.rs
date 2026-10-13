use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]

pub struct Tweet {
    pub id: Uuid,
    pub author_id: Uuid,
    pub content: String,
    pub created_at: OffsetDateTime,
    pub author_username: String,
    pub author_display_name: String,
    pub author_avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTweet {
    pub author_id: Uuid,
    pub content: String,
}

pub async fn get_all(pool: &PgPool) -> Result<Vec<Tweet>, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#"
        SELECT tweets.*, users.username AS author_username, users.display_name AS author_display_name, users.avatar_url AS author_avatar_url
        FROM tweets
        JOIN users ON tweets.author_id = users.id
        ORDER BY tweets.created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

pub async fn get_by_username(pool: &PgPool, username: &str) -> Result<Vec<Tweet>, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#"
        SELECT tweets.*, users.username AS author_username, users.display_name AS author_display_name, users.avatar_url AS author_avatar_url
        FROM tweets
        JOIN users ON tweets.author_id = users.id
        WHERE users.username = $1
        ORDER BY tweets.created_at DESC
        "#,
        username
    )
    .fetch_all(pool)
    .await
}

// Essentially this function retrieves the tweets from the users that the given user is following.
// It joins the tweets table with the users table to get the author information and also joins with
// the follows table to filter only those tweets from authors that the user is following.
pub async fn get_feed(pool: &PgPool, user_id: Uuid) -> Result<Vec<Tweet>, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#"
    SELECT t.id, t.author_id, t.content, t.created_at,
    u.username AS author_username,
    u.display_name AS author_display_name,
    u.avatar_url AS author_avatar_url
    FROM tweets t
    JOIN users u ON u.id = t.author_id
    JOIN follows f ON f.following_id = t.author_id AND f.follower_id = $1
    ORDER BY t.created_at DESC
    "#,
        user_id
    )
    .fetch_all(pool)
    .await
}

pub async fn create(pool: &PgPool, input: CreateTweet) -> Result<Tweet, sqlx::Error> {
    sqlx::query_as!(
        Tweet,
        r#"
        WITH inserted AS (
        INSERT INTO tweets (author_id, content)
        VALUES ($1, $2)
        RETURNING *
        )
        SELECT i.id, i.author_id, i.content, i.created_at,
        u.username AS author_username,
        u.display_name AS author_display_name,
        u.avatar_url AS author_avatar_url
        FROM inserted i
        JOIN users u ON u.id = i.author_id
        "#,
        input.author_id,
        input.content
    )
    .fetch_one(pool)
    .await
}

pub async fn delete(pool: &PgPool, tweet_id: Uuid) -> Result<Option<Uuid>, sqlx::Error> {
    let row = sqlx::query_scalar!("DELETE FROM tweets WHERE id = $1 RETURNING id", tweet_id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}
