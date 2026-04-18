use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn init_db() -> Result<PgPool, sqlx::Error> {
    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    Ok(pool)
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