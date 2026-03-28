use tracing::{info, warn};

pub async fn collect_trace_logs(trace_id: &str) {
    info!("Collecting OpenTelemetry distributed traces for: {}", trace_id);
    
    // Querying the Jaeger REST API to retrieve distributed span events securely without grpc overhead
    let url = format!("http://localhost:16686/api/traces/{}", trace_id);
    let client = reqwest::Client::new();
    
    match client.get(&url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                info!("OpenTelemetry stack traces located securely within Jaeger.");
            } else {
                warn!("Jaeger REST API returned failure status: {}", res.status());
            }
        },
        Err(e) => warn!("OpenTelemetry Jaeger cluster completely unreachable: {}", e),
    }
}
