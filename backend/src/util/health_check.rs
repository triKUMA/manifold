use deadpool_redis::{
    redis::{cmd, Value},
    Pool,
};
use sqlx::MySqlPool;

use crate::common::Error;

pub async fn database_connection_check(pool: &MySqlPool) -> Result<(), Error> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|err| err.into())
}

pub async fn redis_connection_check(pool: &Pool) -> Result<(), Error> {
    let mut conn = pool.get().await?;

    cmd("SET")
        .arg(&["connection_check", "42"])
        .query_async(&mut conn)
        .await?;

    let get_value: Value = cmd("GET")
        .arg(&["connection_check"])
        .query_async(&mut conn)
        .await?;

    if get_value != Value::Data("42".into()) {
        return Err("Unexpected redis value returned".into());
    }

    Ok(())
}