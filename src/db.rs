use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::errors::AppError;


/// 初始化数据库连接池
pub async fn init_db() -> Result<PgPool, sqlx::Error> {
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
}


pub fn handle_db_error(error: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &error {
        if let Some(code) = db_err.code() {
            if code == "23505" {
                if db_err.message().contains("username") {
                    return AppError::Conflict("username".to_string());
                }
                if db_err.message().contains("email") {
                    return AppError::Conflict("email".to_string());
                }
            }
        }
    }
    AppError::Database(error)
}


#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Executor;

    #[tokio::test]
    async fn test_db_connection() {
        let pool = init_db().await.expect("Failed to initialize the database");
        
        // Test a simple query
        let result = pool.execute("SELECT 1").await;
        assert!(result.is_ok(), "Failed to execute test query");
    }
}