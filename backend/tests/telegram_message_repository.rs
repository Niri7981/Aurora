use aurora::application::telegram_message_service::{SaveTelegramMessage, TelegramMessageService};
use aurora::domain::telegram_message::ContentUrl;
use aurora::infrastructure::database::telegram_message_repository::SaveTelegramMessageOutcome;
use aurora::infrastructure::database::telegram_message_repository::{
    SearchTelegramMessages, TelegramMessageRepository,
};
use chrono::{DateTime, Utc};
use sqlx::postgres::PgPoolOptions;

fn timestamp(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .expect("fixture timestamp should be valid")
        .with_timezone(&Utc)
}

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn saves_one_telegram_message_without_duplicating_its_source() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("PostgreSQL should be available");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migrations should run");

    let mut transaction = pool.begin().await.expect("transaction should begin");
    let content_urls = [ContentUrl {
        url: "https://example.com/jobs/rust".to_string(),
        kind: Some("job_application".to_string()),
    }];
    let fixture = SaveTelegramMessage {
        channel_name: "Example jobs",
        content_text: "A fictional company is hiring a Rust engineer.",
        author_name: Some("Example recruiter"),
        published_at: Some(timestamp("2026-08-07T23:27:00+08:00")),
        external_message_id: Some("42"),
        external_url: Some("https://t.me/example_jobs/42"),
        content_urls: &content_urls,
    };
    let created = TelegramMessageService::save(&mut transaction, fixture)
        .await
        .expect("message should be saved");
    let created = match created {
        SaveTelegramMessageOutcome::Created(message) => message,
        SaveTelegramMessageOutcome::AlreadyExists(_) => panic!("fixture should be new"),
    };

    let duplicate = TelegramMessageService::save(
        &mut transaction,
        SaveTelegramMessage {
            content_text: "An improved extraction of the same fictional role.",
            ..fixture
        },
    )
    .await
    .expect("duplicate source should be detected");
    let duplicate = match duplicate {
        SaveTelegramMessageOutcome::AlreadyExists(message) => message,
        SaveTelegramMessageOutcome::Created(_) => panic!("duplicate should not be created"),
    };
    assert_eq!(duplicate.id, created.id);
    assert_eq!(duplicate.content_text, created.content_text);
    assert_eq!(created.content_urls.len(), 2);
    assert!(
        created
            .content_urls
            .iter()
            .any(|content_url| content_url.url == "https://example.com/jobs/rust")
    );

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM telegram_messages WHERE external_url = $1")
            .bind("https://t.me/example_jobs/42")
            .fetch_one(&mut *transaction)
            .await
            .expect("saved message count should be readable");
    assert_eq!(count, 1);

    let terms = vec!["rust".to_string()];
    let found = TelegramMessageRepository::search(
        &mut transaction,
        SearchTelegramMessages {
            terms: &terms,
            channel_name: Some("Example jobs"),
            starts_at: Some(timestamp("2026-08-01T00:00:00Z")),
            ends_at: Some(timestamp("2026-09-01T00:00:00Z")),
            offset: 0,
            limit: 10,
        },
    )
    .await
    .expect("saved message should be searchable");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, created.id);
    let count = TelegramMessageRepository::count(
        &mut transaction,
        &SearchTelegramMessages {
            terms: &terms,
            channel_name: Some("Example jobs"),
            starts_at: Some(timestamp("2026-08-01T00:00:00Z")),
            ends_at: Some(timestamp("2026-09-01T00:00:00Z")),
            offset: 0,
            limit: 10,
        },
    )
    .await
    .expect("exact match count should be readable");
    assert_eq!(count, 1);

    let after_last_result = TelegramMessageRepository::search(
        &mut transaction,
        SearchTelegramMessages {
            terms: &terms,
            channel_name: Some("Example jobs"),
            starts_at: None,
            ends_at: None,
            offset: 1,
            limit: 10,
        },
    )
    .await
    .expect("an exhausted page should succeed");
    assert!(after_last_result.is_empty());

    let url_terms = vec!["example.com/jobs/rust".to_string()];
    let found_by_url = TelegramMessageRepository::search(
        &mut transaction,
        SearchTelegramMessages {
            terms: &url_terms,
            channel_name: None,
            starts_at: None,
            ends_at: None,
            offset: 0,
            limit: 10,
        },
    )
    .await
    .expect("message URL should be searchable");
    assert_eq!(found_by_url.len(), 1);
    assert_eq!(found_by_url[0].id, created.id);

    let missing_terms = vec!["python".to_string()];
    let missing = TelegramMessageRepository::search(
        &mut transaction,
        SearchTelegramMessages {
            terms: &missing_terms,
            channel_name: Some("Example jobs"),
            starts_at: None,
            ends_at: None,
            offset: 0,
            limit: 10,
        },
    )
    .await
    .expect("unmatched search should succeed");
    assert!(missing.is_empty());

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");
}
