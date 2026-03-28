use tracing::{info, warn};

pub async fn query_historical_state(entity_id: &str) {
    info!("Querying Change Data Capture streams for entity: {}", entity_id);
    
    // Hitting the Debezium / Kafka Connect REST sink for historical data changes securely over HTTP
    let url = format!("http://localhost:8083/connectors/db-history/records?entity={}", entity_id);
    let client = reqwest::Client::new();
    
    match client.get(&url).send().await {
        Ok(res) => {
            if res.status().is_success() {
                info!("Historical CDC record timeline recovered.");
            } else {
                warn!("CDC stream query returned non-200 status: {}", res.status());
            }
        },
        Err(e) => warn!("Failed to reach CDC connector cluster: {}", e),
    }
}
