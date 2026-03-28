use tracing::{info, warn};
use tokio_postgres::NoTls;
use crate::settings::Settings;

pub async fn extract_application_state(state_bucket: &str, identifier: &str) {
    info!("Extracting read-only state from unified data bucket '{}' for entity '{}'", state_bucket, identifier);
    
    // Load external application configurations natively
    let config = Settings::load();
    
    match tokio_postgres::connect(&config.target_app_db_url, NoTls).await {
        Ok((client, connection)) => {
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    warn!("Database connection link dropped securely: {}", e);
                }
            });

            let query = format!("SELECT * FROM {} WHERE id = $1 LIMIT 1", state_bucket);
            
            match client.query(&query, &[&identifier]).await {
                Ok(rows) => info!("Successfully pulled {} universal state snapshots from live {} cluster.", rows.len(), config.target_app_db_url),
                Err(e) => warn!("Universal Data query evaluated correctly but storage execution failed: {}", e),
            }
        },
        Err(e) => {
            warn!("Could not connect to target state data store at {}. Error: {}", config.target_app_db_url, e);
        }
    }
}
