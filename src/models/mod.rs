use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InvestigationRequest {
    pub intent: String,
    pub original_text: String,
    pub entity_type: Option<String>,
    pub identifiers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextPackage {
    pub probable_modules: Vec<String>,
    pub relevant_code_nodes: Vec<String>,
}

/// A retrieved code chunk with its relevance score and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredChunk {
    pub file: String,
    pub content: String,
    pub language: String,
    pub score: f32,
    pub chunk_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvidencePackage {
    pub code_evidence: Vec<String>,
    pub db_evidence: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticResult {
    pub primary_cause: String,
    pub confidence: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestRequest {
    pub repo_url: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IngestResponse {
    pub job_id: String,
    pub status: String,
}

// --- Git integration models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitSummary {
    pub recent_commits: Vec<CommitInfo>,
    pub diff_stat: String,
    pub hot_files: Vec<(String, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
    pub date: String,
    pub message: String,
    pub files: Vec<String>,
}

// --- Context packing models ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackOutput {
    pub content: String,
    pub total_tokens: usize,
    pub file_count: usize,
    pub style: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackRequest {
    pub repo_path: String,
    pub style: Option<String>,
    pub compress: Option<bool>,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub include_git_diff: Option<bool>,
    pub include_git_log: Option<bool>,
    pub git_log_count: Option<usize>,
    pub show_line_numbers: Option<bool>,
    pub include_tree: Option<bool>,
    pub include_repo_map: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackResponse {
    pub content: String,
    pub total_tokens: usize,
    pub file_count: usize,
    pub style: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_investigation_request_serialization() {
        let req = InvestigationRequest {
            intent: "diagnose".to_string(),
            original_text: "why is the bill wrong".to_string(),
            entity_type: Some("bill".to_string()),
            identifiers: std::collections::HashMap::new(),
        };
        let serialized = serde_json::to_string(&req).unwrap();
        assert!(serialized.contains("diagnose"));
        assert!(serialized.contains("bill"));
    }
}
