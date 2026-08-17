use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchMatchMode {
    AllTerms,
    AnyTerms,
}

impl SearchMatchMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AllTerms => "all_terms",
            Self::AnyTerms => "any_terms",
        }
    }
}
