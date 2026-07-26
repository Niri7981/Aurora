use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextDocument {
    pub label: String,
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalContext {
    pub identity_card: Option<ContextDocument>,
    pub current_focus: Option<ContextDocument>,
    pub preferences: Option<ContextDocument>,
    pub privacy_rules: Option<ContextDocument>,
    pub project_contexts: Vec<ContextDocument>,
    pub missing: Vec<PathBuf>,
}
