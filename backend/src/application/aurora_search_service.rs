use std::collections::HashSet;

use chrono::{DateTime, Utc};
use sqlx::PgConnection;

use crate::application::context_gateway::ContextGateway;
use crate::domain::context_pack::{ContextItem, ContextPack};
use crate::domain::telegram_message::TelegramMessage;
use crate::infrastructure::audit_log::AuditLog;
use crate::infrastructure::database::telegram_message_repository::{
    SearchTelegramMessages, TelegramMessageRepository,
};

const MAX_PERSONAL_ITEMS: usize = 6;
const MAX_TELEGRAM_CONTENT_CHARS: usize = 1_600;
const MAX_QUERY_TERMS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchAurora<'a> {
    pub query: &'a str,
    pub purpose: &'a str,
    pub include_personal_context: bool,
    pub include_telegram: bool,
    pub channel_name: Option<&'a str>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub max_results: u32,
}

#[derive(Clone)]
pub struct AuroraSearchService {
    context_gateway: ContextGateway,
    audit_log: AuditLog,
    client: String,
}

impl AuroraSearchService {
    pub fn new(
        context_gateway: ContextGateway,
        audit_log: AuditLog,
        client: impl Into<String>,
    ) -> Self {
        Self {
            context_gateway,
            audit_log,
            client: client.into(),
        }
    }

    pub async fn search(
        &self,
        connection: &mut PgConnection,
        request: SearchAurora<'_>,
    ) -> Result<ContextPack, String> {
        let result = self.search_inner(connection, request).await;
        match result {
            Ok(pack) => {
                self.audit_log
                    .append_success(&self.client, "search_aurora", &pack)?;
                Ok(pack)
            }
            Err(error) => {
                self.audit_log.append_failure(
                    &self.client,
                    "search_aurora",
                    request.purpose,
                    Some(request.query),
                    &error,
                )?;
                Err(error)
            }
        }
    }

    async fn search_inner(
        &self,
        connection: &mut PgConnection,
        request: SearchAurora<'_>,
    ) -> Result<ContextPack, String> {
        let personal_limit = if request.include_personal_context && request.include_telegram {
            (request.max_results / 3)
                .max(1)
                .min(MAX_PERSONAL_ITEMS as u32) as usize
        } else {
            (request.max_results as usize).min(MAX_PERSONAL_ITEMS)
        };
        let mut personal_pack = if request.include_personal_context {
            self.context_gateway.search_personal_context_unlogged(
                request.query,
                request.purpose,
                Some(personal_limit),
            )?
        } else {
            empty_pack(request, &self.client)
        };

        let mut items = Vec::new();
        if request.include_telegram {
            let terms = query_terms(request.query);
            let telegram_limit = request
                .max_results
                .saturating_sub(personal_pack.items.len() as u32);
            let messages = TelegramMessageRepository::search(
                connection,
                SearchTelegramMessages {
                    terms: &terms,
                    channel_name: request.channel_name,
                    starts_at: request.starts_at,
                    ends_at: request.ends_at,
                    limit: telegram_limit,
                },
            )
            .await
            .map_err(|error| format!("failed to search Telegram messages: {error}"))?;
            items.extend(messages.into_iter().map(telegram_context_item));
        }
        items.append(&mut personal_pack.items);
        items.truncate(request.max_results as usize);

        Ok(ContextPack {
            purpose: request.purpose.to_string(),
            query: Some(request.query.to_string()),
            client: self.client.clone(),
            access: "read-only, source-attributed, minimum-necessary disclosure".to_string(),
            items,
            omissions: personal_pack.omissions,
        })
    }
}

fn empty_pack(request: SearchAurora<'_>, client: &str) -> ContextPack {
    ContextPack {
        purpose: request.purpose.to_string(),
        query: Some(request.query.to_string()),
        client: client.to_string(),
        access: "read-only".to_string(),
        items: Vec::new(),
        omissions: Vec::new(),
    }
}

fn telegram_context_item(message: TelegramMessage) -> ContextItem {
    let mut metadata = vec![format!("Channel: {}", message.channel_name)];
    if let Some(author_name) = &message.author_name {
        metadata.push(format!("Author: {author_name}"));
    }
    if let Some(published_at) = message.published_at {
        metadata.push(format!("Published: {}", published_at.to_rfc3339()));
    }
    if let Some(external_url) = &message.external_url {
        metadata.push(format!("External URL: {external_url}"));
    }
    let (content, truncated) = truncate_chars(&message.content_text, MAX_TELEGRAM_CONTENT_CHARS);
    metadata.push(String::new());
    metadata.push(content);

    ContextItem {
        category: "telegram_message".to_string(),
        label: format!("Telegram · {}", message.channel_name),
        source: format!("aurora://telegram/messages/{}", message.id),
        content: metadata.join("\n"),
        truncated,
    }
}

fn query_terms(query: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut terms = Vec::new();
    for term in query.split(|character: char| {
        !(character.is_alphanumeric()
            || character == '_'
            || matches!(character, '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}'))
    }) {
        let term = term.trim().to_lowercase();
        if term.chars().count() >= 2 && seen.insert(term.clone()) {
            terms.push(term);
            if terms.len() == MAX_QUERY_TERMS {
                break;
            }
        }
    }
    terms
}

fn truncate_chars(content: &str, max_chars: usize) -> (String, bool) {
    let mut characters = content.chars();
    let truncated = characters.by_ref().take(max_chars).collect::<String>();
    let was_truncated = characters.next().is_some();
    (truncated, was_truncated)
}

#[cfg(test)]
mod tests {
    use super::query_terms;

    #[test]
    fn query_terms_support_ascii_and_cjk_queries() {
        assert_eq!(
            query_terms("payments + AI，招聘"),
            vec!["payments", "ai", "招聘"]
        );
    }
}
