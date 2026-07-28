use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub mod import_batch_repository;

pub async fn connect_and_migrate() -> Result<PgPool, String> {
    let database_url = required_database_url(std::env::var("DATABASE_URL").ok())?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|error| format!("failed to connect to PostgreSQL: {error}"))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|error| format!("failed to run PostgreSQL migrations: {error}"))?;

    Ok(pool)
}

fn required_database_url(value: Option<String>) -> Result<String, String> {
    match value.map(|value| value.trim().to_string()) {
        Some(value) if !value.is_empty() => Ok(value),
        _ => Err(
            "DATABASE_URL is not set; copy .env.example to .env and configure PostgreSQL"
                .to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::required_database_url;

    #[test]
    fn requires_a_non_empty_database_url() {
        assert!(required_database_url(None).is_err());
        assert!(required_database_url(Some("   ".to_string())).is_err());
        assert_eq!(
            required_database_url(Some(" postgresql://localhost/aurorapulse ".to_string())),
            Ok("postgresql://localhost/aurorapulse".to_string())
        );
    }
}
