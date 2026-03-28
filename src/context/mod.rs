use crate::models::{InvestigationRequest, ContextPackage};
use tracing::info;

pub async fn resolve_context(request: &InvestigationRequest) -> ContextPackage {
    info!("Resolving context for intent: {}", request.intent);
    ContextPackage {
        probable_modules: vec!["stub_module".to_string()],
        relevant_code_nodes: vec![],
    }
}
