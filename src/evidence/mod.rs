use crate::models::{ContextPackage, EvidencePackage};
use tracing::info;
use crate::gatherers::{db_adapter, opentelemetry_ingest};
use crate::settings::Settings;

pub async fn collect_evidence(context: &ContextPackage) -> EvidencePackage {
    info!("Executing evidence gathering protocols for context paths: {:?}", context.relevant_code_nodes);

    let mut code_evidence = Vec::new();
    for node in &context.relevant_code_nodes {
        info!("Forwarding Semantic AST chunk to Intelligence Evaluator...");
        code_evidence.push(node.clone());
        
        if node.starts_with("QdrantNode_") {
            opentelemetry_ingest::collect_trace_logs(node).await;
        }
    }

    // Only attempt DB evidence if real credentials are configured
    let config = Settings::load();
    let db_evidence = if !config.target_app_db_url.is_empty()
        && !config.target_app_db_url.contains("customer_target_db")
        && !config.target_app_db_url.contains("data_exporter")
    {
        info!("Target application DB configured — gathering live database evidence...");
        Some(db_adapter::extract_application_state("core_metadata", "runtime").await)
    } else {
        info!("No target application DB configured — proceeding with code evidence only.");
        None
    };

    EvidencePackage {
        code_evidence,
        db_evidence,
    }
}
