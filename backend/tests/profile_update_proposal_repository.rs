use aurora::domain::profile_update_proposal::{ProfileUpdateStatus, ProfileUpdateTarget};
use aurora::infrastructure::database::profile_update_proposal_repository::{
    NewProfileUpdateProposal, ProfileUpdateProposalRepository,
};
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
#[ignore = "requires the local Docker PostgreSQL database"]
async fn creates_finds_and_lists_only_pending_profile_update_proposals() {
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
    let pending = ProfileUpdateProposalRepository::create(
        &mut transaction,
        NewProfileUpdateProposal {
            target: ProfileUpdateTarget::CurrentFocus,
            base_sha256: &"a".repeat(64),
            proposed_content: "# Current focus\n\nBuild the proposal repository.\n",
            reason: "The user started the repository task",
            proposed_by: "hermes",
        },
    )
    .await
    .expect("pending proposal should be created");

    assert_eq!(pending.target, ProfileUpdateTarget::CurrentFocus);
    assert_eq!(pending.status, ProfileUpdateStatus::Pending);
    assert_eq!(pending.proposed_by, "hermes");
    assert!(pending.decided_at.is_none());

    let rejected = ProfileUpdateProposalRepository::create(
        &mut transaction,
        NewProfileUpdateProposal {
            target: ProfileUpdateTarget::Preferences,
            base_sha256: &"b".repeat(64),
            proposed_content: r#"{"response_style":"brief"}"#,
            reason: "The agent inferred a response preference",
            proposed_by: "hermes",
        },
    )
    .await
    .expect("second proposal should be created");

    let rejected = ProfileUpdateProposalRepository::transition_pending(
        &mut transaction,
        rejected.id,
        ProfileUpdateStatus::Rejected,
    )
    .await
    .expect("test proposal should be marked rejected")
    .expect("pending proposal should transition");
    assert_eq!(rejected.status, ProfileUpdateStatus::Rejected);
    assert!(rejected.decided_at.is_some());

    let found =
        ProfileUpdateProposalRepository::find_by_id_for_update(&mut transaction, pending.id)
            .await
            .expect("proposal lookup should succeed")
            .expect("created proposal should be found");
    assert_eq!(found, pending);

    let missing = ProfileUpdateProposalRepository::find_by_id(&mut transaction, uuid::Uuid::nil())
        .await
        .expect("missing proposal lookup should succeed");
    assert!(missing.is_none());

    let pending_proposals = ProfileUpdateProposalRepository::list_pending(&mut transaction)
        .await
        .expect("pending proposals should be listed");
    assert!(pending_proposals.iter().any(|item| item == &pending));
    assert!(pending_proposals.iter().all(|item| item.id != rejected.id));

    let deleted = ProfileUpdateProposalRepository::delete_pending(&mut transaction, pending.id)
        .await
        .expect("pending proposal deletion should succeed");
    assert!(deleted);
    let retained = ProfileUpdateProposalRepository::delete_pending(&mut transaction, rejected.id)
        .await
        .expect("terminal proposal deletion should be checked");
    assert!(!retained);

    transaction
        .rollback()
        .await
        .expect("transaction should roll back");

    let remaining: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM profile_update_proposals WHERE id = $1")
            .bind(pending.id)
            .fetch_one(&pool)
            .await
            .expect("rolled back proposal count should be readable");
    assert_eq!(remaining, 0);
}
