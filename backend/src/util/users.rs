use crate::{common::MFResult, models::users::*};
use sqlx::MySqlPool;
use uuid::Uuid;

pub async fn get_users(db_pool: &MySqlPool) -> MFResult<Vec<User>> {
    let users = sqlx::query_as!(
        DbUser,
        "SELECT as_uuid(id) AS id, username, display_name, first_name, last_name, created_at, updated_at FROM users ORDER BY id"
    )
    .fetch_all(db_pool)
    .await
    .map(|db_users| {
        db_users
            .iter()
            .map(|db_user| db_user.clone().into())
            .collect()
    })?;

    Ok(users)
}

pub async fn get_user(id: String, db_pool: &MySqlPool) -> MFResult<Option<User>> {
    let user = sqlx::query_as!(
        DbUser,
        "SELECT as_uuid(id) AS id, username, display_name, first_name, last_name, created_at, updated_at FROM users WHERE id = as_bin(?)",
        id
    )
    .fetch_optional(db_pool)
    .await
    .map(|db_user| db_user.map(|db_user| db_user.into()))?;

    Ok(user)
}

pub async fn add_user(new_user: NewUser, db_pool: &MySqlPool) -> MFResult<String> {
    let user_id = Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO users (id, username, display_name) VALUES (as_bin(?), ?, ?)",
        user_id.clone(),
        new_user.username,
        new_user.username
    )
    .execute(db_pool)
    .await?;

    Ok(user_id)
}

pub async fn delete_user(id: String, db_pool: &MySqlPool) -> MFResult<()> {
    sqlx::query!("DELETE FROM users WHERE id = as_bin(?)", id)
        .execute(db_pool)
        .await?;

    Ok(())
}
