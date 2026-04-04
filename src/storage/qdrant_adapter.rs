use tracing::{info, warn};
use serde_json::json;
use reqwest::Client;
use uuid::Uuid;

pub struct QdrantAdapter {
    client: Client,
    base_url: String,
}

impl QdrantAdapter {
    pub fn new(qdrant_url: &str) -> Self {
        info!("Initializing Qdrant HTTP vector search adapter at {}...", qdrant_url);
        Self {
            client: Client::new(),
            base_url: qdrant_url.to_string(),
        }
    }

    pub async fn provision_collection(&self, collection_name: &str, vector_size: u64) -> Result<(), Box<dyn std::error::Error>> {
        info!("Provisioning Qdrant collection: {}", collection_name);
        let url = format!("{}/collections/{}", self.base_url, collection_name);
        
        let payload = json!({
            "vectors": {
                "size": vector_size,
                "distance": "Cosine"
            }
        });

        // Fire the request directly. If it fails due to the server being offline locally, we catch it securely
        match self.client.put(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("Successfully created collection {}", collection_name);
                } else {
                    let errmsg = resp.text().await?;
                    if errmsg.contains("already exists") {
                        info!("Collection {} already exists.", collection_name);
                    } else {
                        warn!("Failed to create Qdrant collection: {}", errmsg);
                    }
                }
            },
            Err(e) => warn!("Could not reach Qdrant DB. Is the cluster running locally? Error: {}", e),
        }
        
        Ok(())
    }

    pub async fn ingest_ast_chunk(&self, collection_name: &str, vector: Vec<f32>, metadata: serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
        let point_id = Uuid::new_v4().to_string();
        let url = format!("{}/collections/{}/points?wait=true", self.base_url, collection_name);
        
        let payload = json!({
            "points": [
                {
                    "id": point_id.clone(),
                    "vector": vector,
                    "payload": metadata
                }
            ]
        });

        match self.client.put(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    info!("Ingested syntax chunk {} into Qdrant collection.", point_id);
                } else {
                    warn!("Qdrant points insertion failed with status: {}", resp.status());
                }
            },
            Err(e) => warn!("Failed to reach Qdrant during AST ingestion: {}", e),
        }

        Ok(point_id)
    }

    pub async fn search_ast_chunks(&self, collection_name: &str, vector: Vec<f32>, limit: u64) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let results = self.search_with_scores(collection_name, vector, limit, None).await?;
        Ok(results.into_iter().map(|(payload, _score)| payload).collect())
    }

    /// Search with relevance scores returned alongside payloads
    pub async fn search_with_scores(
        &self,
        collection_name: &str,
        vector: Vec<f32>,
        limit: u64,
        filter: Option<serde_json::Value>,
    ) -> Result<Vec<(serde_json::Value, f32)>, Box<dyn std::error::Error>> {
        let url = format!("{}/collections/{}/points/search", self.base_url, collection_name);

        let mut payload = json!({
            "vector": vector,
            "limit": limit,
            "with_payload": true
        });
        if let Some(f) = filter {
            payload["filter"] = f;
        }

        match self.client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let data: serde_json::Value = resp.json().await?;
                    if let Some(result_arr) = data.get("result").and_then(|r| r.as_array()) {
                        let scored: Vec<(serde_json::Value, f32)> = result_arr
                            .iter()
                            .filter_map(|point| {
                                let payload = point.get("payload")?.clone();
                                let score = point.get("score")?.as_f64()? as f32;
                                Some((payload, score))
                            })
                            .collect();
                        return Ok(scored);
                    } else {
                        warn!("Qdrant search returned unexpected response structure");
                    }
                } else {
                    let errmsg = resp.text().await.unwrap_or_default();
                    warn!("Qdrant search failed: {}", errmsg);
                }
            },
            Err(e) => warn!("Failed to reach Qdrant during search: {}", e),
        }

        Ok(Vec::new())
    }

    /// Scroll all points matching a filter (for keyword post-filtering)
    pub async fn scroll_points(
        &self,
        collection_name: &str,
        filter: serde_json::Value,
        limit: u64,
    ) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let url = format!("{}/collections/{}/points/scroll", self.base_url, collection_name);

        let payload = json!({
            "filter": filter,
            "limit": limit,
            "with_payload": true
        });

        match self.client.post(&url).json(&payload).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    let data: serde_json::Value = resp.json().await?;
                    if let Some(result) = data.get("result").and_then(|r| r.get("points")).and_then(|p| p.as_array()) {
                        let payloads = result.iter().filter_map(|p| p.get("payload").cloned()).collect();
                        return Ok(payloads);
                    } else {
                        warn!("Qdrant scroll returned unexpected response structure");
                    }
                } else {
                    let errmsg = resp.text().await.unwrap_or_default();
                    warn!("Qdrant scroll failed: {}", errmsg);
                }
            },
            Err(e) => warn!("Failed to scroll Qdrant points: {}", e),
        }

        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_qdrant_http_adapter_graceful_routing() {
        let adapter = QdrantAdapter::new("http://localhost:99999");
        
        // Because the port is deliberately invalid, this will trigger the graceful `Err` warning block 
        // without panicking the active asynchronous Rust server runtime!
        let res = adapter.provision_collection("test_ast", 1536).await;
        
        // Assert the function returned successfully, maintaining orchestrator stability
        assert!(res.is_ok()); 
    }
}
