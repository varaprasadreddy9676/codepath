use crate::models::{InvestigationRequest, ContextPackage};
use tracing::info;

pub async fn resolve_context(request: &InvestigationRequest) -> ContextPackage {
    info!("Resolving context structural vectors for intent: {}", request.intent);
    
    let mut probable_modules = Vec::new();
    let mut relevant_code_nodes = Vec::new();

    // Map intents precisely to the codebase modules via predefined knowledge routes
    match request.intent.as_str() {
        "technical_diagnosis" => {
            probable_modules.push("core_exceptions".to_string());
            probable_modules.push("infrastructure_routing".to_string());
        },
        "business_behavior_explanation" => {
            probable_modules.push("billing_service".to_string());
            probable_modules.push("pricing_rules".to_string());
        },
        "visibility_explanation" => {
            probable_modules.push("auth_filters".to_string());
            probable_modules.push("worklist_queries".to_string());
        },
        "state_transition_explanation" => {
            probable_modules.push("workflow_engine".to_string());
            probable_modules.push("state_machine".to_string());
        },
        _ => {
            probable_modules.push("global_search".to_string());
        }
    }

    // Forward extracted IDs into vector / lexical lookups
    if let Some(id) = request.identifiers.get("extracted_id") {
        info!("Valid identifier '{}' detected. Triggering simulated Qdrant vector retrieval.", id);
        // Native Qdrant lookup goes here. 
        relevant_code_nodes.push(format!("QdrantNode_{}", id));
    }

    ContextPackage {
        probable_modules,
        relevant_code_nodes,
    }
}
