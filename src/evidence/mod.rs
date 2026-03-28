use crate::models::{ContextPackage, EvidencePackage};
use tracing::info;

pub async fn collect_evidence(context: &ContextPackage) -> EvidencePackage {
    info!("Collecting evidence mapping to modules...");
    EvidencePackage {
        code_evidence: vec![],
        db_evidence: None,
    }
}
