use aurora::domain::profile_update_proposal::{
    ProfileUpdateProposal, ProfileUpdateStatus, ProfileUpdateTarget,
};
use sqlx::postgres::PgPoolOptions;

type ProposalRow = (
    uuid::Uuid,
    String,
    String,
    String,
    String,
    String,
    String,
    chrono::DateTime<chrono::Utc>,
    Option<chrono::DateTime<chrono::Utc>>,
);

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn migration_creates_pending_profile_update_proposals_with_guardrails() {
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

    let proposed_content = "# Current focus\n\nShip controlled profile updates.\n";
    let mut transaction = pool.begin().await.expect("transaction should begin");
    let row = sqlx::query_as::<_, ProposalRow>(
        r#"
        INSERT INTO profile_update_proposals (
            target,
            base_sha256,
            proposed_content,
            reason,
            proposed_by
        )
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
            id,
            target,
            base_sha256,
            proposed_content,
            reason,
            proposed_by,
            status,
            created_at,
            decided_at
        "#,
    )
    .bind("current_focus")
    .bind("a".repeat(64))
    .bind(proposed_content)
    .bind("The user changed the next product milestone")
    .bind("hermes")
    .fetch_one(&mut *transaction)
    .await
    .expect("pending proposal should be inserted");

    let proposal = ProfileUpdateProposal {
        id: row.0,
        target: ProfileUpdateTarget::try_from(row.1.as_str())
            .expect("stored target should be supported"),
        base_sha256: row.2,
        proposed_content: row.3,
        reason: row.4,
        proposed_by: row.5,
        status: ProfileUpdateStatus::try_from(row.6.as_str())
            .expect("stored status should be supported"),
        created_at: row.7,
        decided_at: row.8,
    };

    assert_eq!(proposal.target, ProfileUpdateTarget::CurrentFocus);
    assert_eq!(proposal.proposed_content, proposed_content);
    assert_eq!(proposal.status, ProfileUpdateStatus::Pending);
    assert!(proposal.decided_at.is_none());

    let protected_target = sqlx::query(
        r#"
        INSERT INTO profile_update_proposals (
            target,
            base_sha256,
            proposed_content,
            reason,
            proposed_by
        )
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind("privacy_rules")
    .bind("b".repeat(64))
    .bind("{}")
    .bind("Agent attempted to change its own access policy")
    .bind("hermes")
    .execute(&mut *transaction)
    .await
    .expect_err("privacy rules must not be a proposal target");

    assert_eq!(
        protected_target
            .as_database_error()
            .and_then(|error| error.constraint()),
        Some("profile_update_proposals_target_check")
    );

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");

    let mut transaction = pool.begin().await.expect("transaction should begin");
    let undecided_applied_proposal = sqlx::query(
        r#"
        INSERT INTO profile_update_proposals (
            target,
            base_sha256,
            proposed_content,
            reason,
            proposed_by,
            status
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind("identity_card")
    .bind("c".repeat(64))
    .bind("# Identity\nUpdated identity")
    .bind("A claimed identity change needs user review")
    .bind("hermes")
    .bind("applied")
    .execute(&mut *transaction)
    .await
    .expect_err("an applied proposal must record when the user decided it");

    assert_eq!(
        undecided_applied_proposal
            .as_database_error()
            .and_then(|error| error.constraint()),
        Some("profile_update_proposals_decision_state_check")
    );

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");
}
