use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i32,
    pub is_admin: bool,
    pub active: bool,
    pub thoi_vang: i32,
    pub vnd: i32,
    pub tongnap: i32,
    pub server_login: i32,
    pub last_time_login: DateTime<Utc>,
    pub last_time_logout: DateTime<Utc>,
    pub reward: Option<String>,
    pub ban: bool,
}
impl User {
    pub async fn find_by_credentials(
        pool: &sqlx::MySqlPool,
        username: &str,
        password: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM account WHERE username = ? AND password = ? LIMIT 1",
        )
        .bind(username)
        .bind(password)
        .fetch_optional(pool)
        .await
    }
    pub async fn update_logout_time(
        pool: &sqlx::MySqlPool,
        user_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE account SET last_time_logout = NOW() WHERE id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update_login_time(
        pool: &sqlx::MySqlPool,
        user_id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE account SET last_time_login = NOW() WHERE id = ?")
            .bind(user_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
