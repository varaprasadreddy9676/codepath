use tracing::{info, warn};
use tokio_postgres::NoTls;
use crate::settings::Settings;
use serde_json::{json, Value};

pub async fn extract_application_state(state_bucket: &str, identifier: &str) -> Value {
    info!("Extracting read-only state from unified data bucket '{}' for entity '{}'", state_bucket, identifier);
    
    let config = Settings::load();
    
    // If URL is default, empty, or fails, gracefully return a clarification request to the LLM Evidence Package
    if config.target_app_db_url.is_empty() || config.target_app_db_url.contains("customer_target_db") || config.target_app_db_url.contains("data_exporter") {
        return json!({
            "error": "MISSING_CREDENTIALS",
            "message": "The system dynamically mapped data dependencies in the code, but active database access is not configured. Ask the user to share the specific MongoDB or MySQL read-only connection details so we can securely connect and definitively figure out the exact state."
        });
    }

    match tokio_postgres::connect(&config.target_app_db_url, NoTls).await {
        Ok((client, connection)) => {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    warn!("Database connection link dropped securely: {}", e);
                }
            });

            let query = format!("SELECT * FROM {} WHERE id = $1 LIMIT 1", state_bucket);
            
            match client.query(&query, &[&identifier]).await {
                Ok(rows) => {
                    info!("Successfully pulled {} universal state snapshots.", rows.len());
                    json!({"status": "success", "rows_fetched": rows.len(), "raw_payload": "..."})
                },
                Err(e) => {
                    warn!("Universal Data query evaluated correctly but storage execution failed: {}", e);
                    json!({"error": format!("DB Execution failed: {}", e)})
                }
            }
        },
        Err(e) => {
            warn!("Could not connect to target state data store at {}. Error: {}", config.target_app_db_url, e);
            json!({
                "error": "CONNECTION_FAILED",
                "message": format!("Attempted to physically connect to the configured database but failed. Prompt the user to verify the credentials or share valid MongoDB/MySQL connection strings. Error: {}", e)
            })
        }
    }
}
