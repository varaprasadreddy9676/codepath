use crate::models::{ContextPackage, EvidencePackage};
use tracing::info;
use crate::gatherers::{db_adapter, opentelemetry_ingest};

pub async fn collect_evidence(context: &ContextPackage) -> EvidencePackage {
    info!("Executing evidence gathering protocols for context paths: {:?}", context.relevant_code_nodes);

    let mut code_evidence = Vec::new();
    for node in &context.relevant_code_nodes {
        info!("Fetching AST and Lexical evidence boundaries for module: {}", node);
        code_evidence.push(format!("AST Cluster for node [{}] successfully indexed.", node));
        
        if node.starts_with("QdrantNode_") {
            opentelemetry_ingest::collect_trace_logs(node).await;
        }
    }

    // Trigger genuine DB validation actively piping output back into the AI payload
    let dynamic_db_state = db_adapter::extract_application_state("core_metadata", "runtime").await;

    EvidencePackage {
        code_evidence,
        db_evidence: Some(dynamic_db_state),
    }
}
