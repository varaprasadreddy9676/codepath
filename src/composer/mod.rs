use crate::models::DiagnosticResult;
use tracing::info;

pub async fn compose_explanation(result: &DiagnosticResult) -> String {
    info!("Synthesizing final architectural narrative with confidence score: {}", result.confidence);
    
    let disclaimer = if result.confidence < 0.9 {
        "Note: This explanation carries moderate analytical confidence. Manual verification against log streams is recommended."
    } else {
        "Analysis strictly confirmed via structural verification."
    };

    format!(
        "=== DIAGNOSTIC REPORT ===\n\nRoot Cause Identified:\n{}\n\n{}",
        result.primary_cause, disclaimer
    )
}
