use std::io::{self, Write};

use uuid::Uuid;

use crate::application::profile_update_proposal_service::{
    ApplyProfileUpdateOutcome, ProfileUpdateProposalService,
};
use crate::config;
use crate::domain::context::{ContextDocument, LocalContext};
use crate::domain::disclosure::DisclosurePolicy;
use crate::domain::profile_update_proposal::{ProfileUpdateProposal, ProfileUpdateStatus};
use crate::infrastructure::database;
use crate::infrastructure::{audit_log::AuditLog, local_context};

const USAGE: &str = "Aurora is a local personal memory layer for authorized MCP clients.\n\nUsage:\n  aurora serve [workspace]                         Start the stdio MCP server\n  aurora init [workspace]                          Create missing local identity files\n  aurora preview [workspace]                       Preview externally disclosable context\n  aurora audit [workspace]                         Show recent MCP context access\n  aurora profile approve <proposal-id> [workspace] Review and apply a proposal as the local user\n  aurora help                                      Show this help";

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Serve(Option<String>),
    Init(Option<String>),
    Preview(Option<String>),
    Audit(Option<String>),
    ProfileApprove {
        proposal_id: Uuid,
        workspace: Option<String>,
    },
    Help,
}

pub fn run_args(args: impl IntoIterator<Item = String>) -> Result<(), String> {
    match parse_args(args)? {
        Command::Serve(workspace) => crate::run_mcp(config::load_config(workspace)?),
        Command::Init(workspace) => {
            let config = config::load_config(workspace)?;
            println!(
                "{}",
                render_init_report(&local_context::init_files(&config)?)
            );
            Ok(())
        }
        Command::Preview(workspace) => {
            let config = config::load_config(workspace)?;
            let context = local_context::load(&config)?;
            println!("{}", render_preview(&context));
            Ok(())
        }
        Command::Audit(workspace) => {
            let config = config::load_config(workspace)?;
            let audit_log = AuditLog::new(config.aurora_home.join("audit/mcp.jsonl"));
            println!("{}", audit_log.render_recent(20)?);
            Ok(())
        }
        Command::ProfileApprove {
            proposal_id,
            workspace,
        } => approve_profile_update(config::load_config(workspace)?, proposal_id),
        Command::Help => {
            println!("{USAGE}");
            Ok(())
        }
    }
}

fn approve_profile_update(config: config::AppConfig, proposal_id: Uuid) -> Result<(), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("failed to start profile approval runtime: {error}"))?;
    runtime.block_on(async move {
        let pool = database::connect_and_migrate().await?;
        let service = ProfileUpdateProposalService::new(config, "local-user");
        let proposal = {
            let mut connection = pool.acquire().await.map_err(|error| {
                format!("failed to acquire PostgreSQL connection: {error}")
            })?;
            service
                .find_by_id(&mut connection, proposal_id)
                .await?
                .ok_or_else(|| format!("profile update proposal not found: {proposal_id}"))?
        };
        if proposal.status != ProfileUpdateStatus::Pending {
            return Err(format!(
                "profile update proposal is already {}",
                proposal.status.as_str()
            ));
        }

        println!("{}", render_profile_update_approval(&proposal));
        print!("Apply this proposal? Type y to confirm [y/N]: ");
        io::stdout()
            .flush()
            .map_err(|error| format!("failed to show approval prompt: {error}"))?;
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .map_err(|error| format!("failed to read approval response: {error}"))?;
        if !is_yes(&answer) {
            println!("Cancelled. The proposal remains pending and no profile file was changed.");
            return Ok(());
        }

        match service.apply(&pool, proposal_id).await? {
            ApplyProfileUpdateOutcome::Applied(proposal) => {
                println!(
                    "Applied proposal {} to {}.",
                    proposal.id,
                    proposal.target.as_str()
                );
                Ok(())
            }
            ApplyProfileUpdateOutcome::Stale(proposal) => Err(format!(
                "proposal {} is stale because the profile changed after it was created; nothing was overwritten",
                proposal.id
            )),
            ApplyProfileUpdateOutcome::NotPending(proposal) => Err(format!(
                "proposal {} became {} before it could be applied",
                proposal.id,
                proposal.status.as_str()
            )),
            ApplyProfileUpdateOutcome::NotFound => Err(format!(
                "profile update proposal no longer exists: {proposal_id}"
            )),
        }
    })
}

fn render_profile_update_approval(proposal: &ProfileUpdateProposal) -> String {
    format!(
        "Profile update proposal\n\nProposal: {}\nTarget: {}\nProposed by: {}\nCreated at: {}\nReason: {}\n\nComplete replacement content:\n------------------------------\n{}\n------------------------------\n\nAurora will recheck the current file version before writing.",
        proposal.id,
        proposal.target.as_str(),
        proposal.proposed_by,
        proposal.created_at.to_rfc3339(),
        proposal.reason,
        proposal.proposed_content
    )
}

fn is_yes(answer: &str) -> bool {
    answer.trim().eq_ignore_ascii_case("y")
}

fn render_init_report(report: &local_context::InitReport) -> String {
    let mut output = String::from("Context files ready.\n");

    if !report.created.is_empty() {
        output.push_str("\nCreated:\n");
        for path in &report.created {
            output.push_str(&format!("- {}\n", path.display()));
        }
    }

    if !report.existing.is_empty() {
        output.push_str("\nAlready existed:\n");
        for path in &report.existing {
            output.push_str(&format!("- {}\n", path.display()));
        }
    }

    output.push_str("\nEdit identity-card.md and current-focus.md with your real information.");
    output
}

fn render_preview(context: &LocalContext) -> String {
    let disclosure_policy = DisclosurePolicy::from_context(context);
    let mut output = String::from(
        "External context preview\nPolicy: read-only, minimum-necessary disclosure\n\n",
    );

    render_external_document(&mut output, &context.identity_card, &disclosure_policy);
    render_external_document(&mut output, &context.current_focus, &disclosure_policy);
    render_external_document(&mut output, &context.preferences, &disclosure_policy);

    if context.project_contexts.is_empty() {
        output.push_str("Project Context: not found\n\n");
    } else {
        for document in &context.project_contexts {
            render_external_document(&mut output, &Some(document.clone()), &disclosure_policy);
        }
    }

    if !context.missing.is_empty() {
        output.push_str("Missing optional files:\n");
        for path in &context.missing {
            output.push_str(&format!("- {}\n", path.display()));
        }
    }

    output.trim_end().to_string()
}

fn render_external_document(
    output: &mut String,
    document: &Option<ContextDocument>,
    disclosure_policy: &DisclosurePolicy,
) {
    if let Some(document) = document {
        output.push_str(&format!(
            "{}: {}\n",
            document.label,
            document.path.display()
        ));
        let filtered = disclosure_policy.filter_external(&document.content);
        output.push_str(&filtered.content);
        if filtered.omitted_line_count > 0 {
            output.push_str(&format!(
                "\n[{} line(s) omitted by privacy policy]",
                filtered.omitted_line_count
            ));
        }
        output.push_str("\n\n");
    }
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Ok(Command::Help);
    };

    if matches!(command.as_str(), "help" | "-h" | "--help") {
        reject_extra(&mut args, "help")?;
        return Ok(Command::Help);
    }

    if command == "profile" {
        let subcommand = args
            .next()
            .ok_or_else(|| format!("profile requires a subcommand\n\n{USAGE}"))?;
        if subcommand != "approve" {
            return Err(format!("unknown profile command `{subcommand}`\n\n{USAGE}"));
        }
        let raw_proposal_id = args
            .next()
            .ok_or_else(|| format!("profile approve requires a proposal UUID\n\n{USAGE}"))?;
        let proposal_id = Uuid::parse_str(&raw_proposal_id)
            .map_err(|error| format!("proposal-id must be a UUID: {error}"))?;
        let workspace = args.next();
        reject_extra(&mut args, "profile approve")?;
        return Ok(Command::ProfileApprove {
            proposal_id,
            workspace,
        });
    }

    let workspace = args.next();
    reject_extra(&mut args, &command)?;
    match command.as_str() {
        "serve" => Ok(Command::Serve(workspace)),
        "init" => Ok(Command::Init(workspace)),
        "preview" => Ok(Command::Preview(workspace)),
        "audit" => Ok(Command::Audit(workspace)),
        _ => Err(format!("unknown command `{command}`\n\n{USAGE}")),
    }
}

fn reject_extra(args: &mut impl Iterator<Item = String>, command: &str) -> Result<(), String> {
    if let Some(extra) = args.next() {
        return Err(format!("{command} received an extra argument: {extra}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Command, is_yes, parse_args};
    use uuid::Uuid;

    fn parse(args: &[&str]) -> Result<Command, String> {
        parse_args(args.iter().map(|value| value.to_string()))
    }

    #[test]
    fn no_arguments_show_help() {
        assert_eq!(parse(&[]), Ok(Command::Help));
    }

    #[test]
    fn parses_serve_with_workspace() {
        assert_eq!(
            parse(&["serve", "/tmp/workspace"]),
            Ok(Command::Serve(Some("/tmp/workspace".to_string())))
        );
    }

    #[test]
    fn parses_local_profile_approval_with_an_optional_workspace() {
        let proposal_id = Uuid::parse_str("27de231d-20fc-49d9-9741-fd60a687bad0").unwrap();
        assert_eq!(
            parse(&[
                "profile",
                "approve",
                "27de231d-20fc-49d9-9741-fd60a687bad0",
                "/tmp/workspace"
            ]),
            Ok(Command::ProfileApprove {
                proposal_id,
                workspace: Some("/tmp/workspace".to_string())
            })
        );
    }

    #[test]
    fn only_y_confirms_a_profile_update() {
        assert!(is_yes("y\n"));
        assert!(is_yes(" Y "));
        assert!(!is_yes(""));
        assert!(!is_yes("yes"));
        assert!(!is_yes("n"));
    }

    #[test]
    fn rejects_unknown_commands_and_extra_arguments() {
        assert!(parse(&["chat"]).unwrap_err().contains("unknown command"));
        assert!(parse(&["audit", ".", "extra"]).is_err());
        assert!(parse(&["profile", "approve"]).is_err());
        assert!(parse(&["profile", "reject", "anything"]).is_err());
    }
}
