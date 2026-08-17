use std::collections::HashSet;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::Serialize;
use sqlx::PgConnection;

use crate::application::context_gateway::ContextGateway;
use crate::domain::context_pack::{ContextItem, ContextOmission, ContextPack};
use crate::domain::telegram_message::{ContentUrl, TelegramMessage};
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
    pub offset: u64,
    pub page_size: u32,
    pub count_only: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct AuroraSearchResponse {
    pub purpose: String,
    pub query: String,
    pub client: String,
    pub access: String,
    pub match_semantics: String,
    pub counts: SearchCounts,
    pub page: SearchPage,
    pub items: Vec<AuroraSearchItem>,
    pub omissions: Vec<ContextOmission>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct SearchCounts {
    pub total_matches: u64,
    pub personal_context: u64,
    pub telegram: u64,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct SearchPage {
    pub count_only: bool,
    pub offset: u64,
    pub page_size: u32,
    pub returned_count: u32,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct AuroraSearchItem {
    pub record_type: String,
    pub stored_record_uri: String,
    pub title: String,
    pub collection_source: CollectionSource,
    pub original_source: Option<OriginalSource>,
    pub attributed_author: Option<String>,
    pub published_at: Option<String>,
    pub content_urls: Vec<ContentUrl>,
    pub content_excerpt: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct CollectionSource {
    pub platform: String,
    pub container_name: String,
}

#[derive(Debug, Clone, Serialize, JsonSchema, PartialEq, Eq)]
pub struct OriginalSource {
    pub platform: String,
    pub url: String,
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
    ) -> Result<AuroraSearchResponse, String> {
        let result = self.search_inner(connection, request).await;
        match result {
            Ok(response) => {
                let returned_sources = response
                    .items
                    .iter()
                    .map(|item| item.stored_record_uri.as_str())
                    .collect::<Vec<_>>();
                let omitted_lines = response
                    .omissions
                    .iter()
                    .map(|omission| omission.line_count)
                    .sum();
                self.audit_log.append_search_success(
                    &self.client,
                    "search_aurora",
                    &response.purpose,
                    &response.query,
                    &returned_sources,
                    omitted_lines,
                )?;
                Ok(response)
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
    ) -> Result<AuroraSearchResponse, String> {
        let mut personal_pack = if request.include_personal_context {
            self.context_gateway
                .search_personal_context_exact_unlogged(
                    request.query,
                    request.purpose,
                    Some(MAX_PERSONAL_ITEMS),
                )?
        } else {
            ContextPack {
                purpose: request.purpose.to_string(),
                query: Some(request.query.to_string()),
                client: self.client.clone(),
                access: "read-only".to_string(),
                items: Vec::new(),
                omissions: Vec::new(),
            }
        };
        let personal_count = personal_pack.items.len() as u64;

        let terms = query_terms(request.query);
        let telegram_search = SearchTelegramMessages {
            terms: &terms,
            channel_name: request.channel_name,
            starts_at: request.starts_at,
            ends_at: request.ends_at,
            offset: 0,
            limit: request.page_size,
        };
        let telegram_count = if request.include_telegram {
            TelegramMessageRepository::count(connection, &telegram_search)
                .await
                .map_err(|error| format!("failed to count Telegram messages: {error}"))?
        } else {
            0
        };
        let total_matches = personal_count.saturating_add(telegram_count);

        let mut items = Vec::new();
        if !request.count_only && request.offset < total_matches {
            let mut remaining = request.page_size as usize;
            if request.offset < personal_count {
                let personal_offset = request.offset as usize;
                let personal_items = std::mem::take(&mut personal_pack.items);
                items.extend(
                    personal_items
                        .into_iter()
                        .skip(personal_offset)
                        .take(remaining)
                        .map(personal_search_item),
                );
                remaining = remaining.saturating_sub(items.len());
            }

            let telegram_offset = request.offset.saturating_sub(personal_count);
            if request.include_telegram && remaining > 0 && telegram_offset < telegram_count {
                let messages = TelegramMessageRepository::search(
                    connection,
                    SearchTelegramMessages {
                        offset: telegram_offset,
                        limit: remaining as u32,
                        ..telegram_search
                    },
                )
                .await
                .map_err(|error| format!("failed to search Telegram messages: {error}"))?;
                items.extend(messages.into_iter().map(telegram_search_item));
            }
        }

        let returned_count = items.len() as u32;
        let next_offset = request.offset.saturating_add(u64::from(returned_count));
        let has_more = !request.count_only && returned_count > 0 && next_offset < total_matches;

        Ok(AuroraSearchResponse {
            purpose: request.purpose.to_string(),
            query: request.query.to_string(),
            client: self.client.clone(),
            access: "read-only, source-attributed, paginated disclosure".to_string(),
            match_semantics: "case-insensitive substring; multiple query terms match any term"
                .to_string(),
            counts: SearchCounts {
                total_matches,
                personal_context: personal_count,
                telegram: telegram_count,
            },
            page: SearchPage {
                count_only: request.count_only,
                offset: request.offset,
                page_size: request.page_size,
                returned_count,
                has_more,
                next_cursor: has_more.then(|| encode_cursor(next_offset)),
            },
            items,
            omissions: personal_pack.omissions,
        })
    }
}

fn personal_search_item(item: ContextItem) -> AuroraSearchItem {
    AuroraSearchItem {
        record_type: item.category,
        stored_record_uri: item.source,
        title: item.label.clone(),
        collection_source: CollectionSource {
            platform: "aurora".to_string(),
            container_name: item.label,
        },
        original_source: None,
        attributed_author: None,
        published_at: None,
        content_urls: Vec::new(),
        content_excerpt: item.content,
        truncated: item.truncated,
    }
}

fn telegram_search_item(message: TelegramMessage) -> AuroraSearchItem {
    let (content_excerpt, truncated) =
        truncate_chars(&message.content_text, MAX_TELEGRAM_CONTENT_CHARS);
    let original_source = message.external_url.as_ref().map(|url| OriginalSource {
        platform: platform_from_url(url),
        url: url.clone(),
    });

    AuroraSearchItem {
        record_type: "telegram_message".to_string(),
        stored_record_uri: format!("aurora://telegram/messages/{}", message.id),
        title: format!("Telegram message in {}", message.channel_name),
        collection_source: CollectionSource {
            platform: "telegram".to_string(),
            container_name: message.channel_name,
        },
        original_source,
        attributed_author: message.author_name,
        published_at: message.published_at.map(|value| value.to_rfc3339()),
        content_urls: message.content_urls,
        content_excerpt,
        truncated,
    }
}

fn platform_from_url(url: &str) -> String {
    url.split_once("://")
        .map(|(_, remainder)| remainder)
        .unwrap_or(url)
        .split('/')
        .next()
        .unwrap_or("unknown")
        .trim_start_matches("www.")
        .to_lowercase()
}

pub fn encode_cursor(offset: u64) -> String {
    format!("v1:{offset}")
}

pub fn decode_cursor(cursor: &str) -> Result<u64, String> {
    let offset = cursor
        .strip_prefix("v1:")
        .ok_or_else(|| "cursor is not an Aurora search cursor".to_string())?
        .parse::<u64>()
        .map_err(|_| "cursor contains an invalid offset".to_string())?;
    if offset > i64::MAX as u64 {
        return Err("cursor offset is too large".to_string());
    }
    Ok(offset)
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
    use super::{decode_cursor, encode_cursor, platform_from_url, query_terms};

    #[test]
    fn query_terms_support_ascii_and_cjk_queries() {
        assert_eq!(
            query_terms("payments + AI，招聘"),
            vec!["payments", "ai", "招聘"]
        );
    }

    #[test]
    fn cursor_round_trips_and_rejects_unrelated_values() {
        assert_eq!(decode_cursor(&encode_cursor(42)).unwrap(), 42);
        assert!(decode_cursor("42").is_err());
        assert!(decode_cursor("v2:42").is_err());
    }

    #[test]
    fn original_source_platform_is_derived_from_its_url() {
        assert_eq!(platform_from_url("https://www.x.com/example/1"), "x.com");
        assert_eq!(
            platform_from_url("https://jobs.example.com/1"),
            "jobs.example.com"
        );
    }
}
