use crate::models::{InvestigationRequest, ContextPackage, EvidencePackage, DiagnosticResult};
use tracing::info;

pub async fn interpret_intent(raw_text: String) -> InvestigationRequest {
    info!("Interpreting intent from: {}", raw_text);
    InvestigationRequest {
        intent: "technical_diagnosis".to_string(),
        entity_type: None,
        identifiers: std::collections::HashMap::new(),
    }
}
