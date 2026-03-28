use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct InvestigationRequest {
    pub intent: String,
    pub entity_type: Option<String>,
    pub identifiers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContextPackage {
    pub probable_modules: Vec<String>,
    pub relevant_code_nodes: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_investigation_request_serialization() {
        let req = InvestigationRequest {
            intent: "diagnose".to_string(),
            entity_type: Some("bill".to_string()),
            identifiers: std::collections::HashMap::new(),
        };
        let serialized = serde_json::to_string(&req).unwrap();
        assert!(serialized.contains("diagnose"));
        assert!(serialized.contains("bill"));
    }
}
