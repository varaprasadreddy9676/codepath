use crate::models::DiagnosticResult;
use tracing::info;

pub async fn compose_explanation(result: &DiagnosticResult) -> String {
    info!("Composing final narrative...");
    format!("Root Cause: {}", result.primary_cause)
}
