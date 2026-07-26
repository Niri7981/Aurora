use serde_json::Value;

use crate::domain::context::LocalContext;

const DEFAULT_REDACTION_MARKERS: [&str; 2] = ["private:", "local-only:"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisclosurePolicy {
    redaction_markers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredContent {
    pub content: String,
    pub omitted_line_count: usize,
}

impl DisclosurePolicy {
    pub fn from_context(context: &LocalContext) -> Self {
        let configured = context
            .privacy_rules
            .as_ref()
            .and_then(|document| serde_json::from_str::<Value>(&document.content).ok())
            .and_then(|value| value.get("redaction_markers").cloned())
            .and_then(|markers| markers.as_array().cloned())
            .map(|markers| {
                markers
                    .into_iter()
                    .filter_map(|marker| {
                        marker.as_str().map(str::trim).map(str::to_ascii_lowercase)
                    })
                    .filter(|marker| !marker.is_empty())
                    .collect::<Vec<_>>()
            })
            .filter(|markers| !markers.is_empty());

        Self {
            redaction_markers: configured.unwrap_or_else(|| {
                DEFAULT_REDACTION_MARKERS
                    .iter()
                    .map(|marker| marker.to_string())
                    .collect()
            }),
        }
    }

    pub fn filter_external(&self, content: &str) -> FilteredContent {
        let mut omitted_line_count = 0;
        let lines = content
            .lines()
            .filter(|line| {
                let lower = line.to_ascii_lowercase();
                let blocked = self
                    .redaction_markers
                    .iter()
                    .any(|marker| lower.contains(marker));
                omitted_line_count += usize::from(blocked);
                !blocked
            })
            .collect::<Vec<_>>();

        FilteredContent {
            content: lines.join("\n"),
            omitted_line_count,
        }
    }
}
