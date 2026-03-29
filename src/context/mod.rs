use crate::models::{InvestigationRequest, ContextPackage};
use tracing::info;
use crate::settings::Settings;
use crate::storage::qdrant_adapter::QdrantAdapter;
use reqwest::Client;
use serde_json::{json, Value};

async fn extract_offline_embeddings(text: &str) -> Vec<f32> {
    let config = Settings::load();
    let client = Client::new();
    let url = config.llm_api_url.replace("/chat/completions", "/embeddings");
    let payload = json!({ "model": config.llm_model, "prompt": text });

    match client.post(&url).json(&payload).send().await {
        Ok(res) => {
            if let Ok(data) = res.json::<Value>().await {
                if let Some(emb) = data.get("embedding").and_then(|e| e.as_array()) {
                    return emb.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                }
            }
        },
        Err(e) => tracing::warn!("Failed to hit LLM Embeddings module offline implicitly: {}", e),
    }
    vec![0.1; 4096] 
}

pub async fn resolve_context(request: &InvestigationRequest) -> ContextPackage {
    info!("Resolving context structural vectors for semantic intent: {}", request.intent);
    
    let probable_modules = Vec::new();
    let mut relevant_code_nodes = Vec::new();

    // 1. Convert intent payload directly into an offline dimensional Vector
    let config = Settings::load();
    let qdrant = QdrantAdapter::new(&config.qdrant_url);
    
    info!("Transcoding user diagnostic string into semantic lookup payload...");
    let semantic_vector = extract_offline_embeddings(&request.intent).await;

    // 2. Perform similarity cosine matching across AST index explicitly
    if let Ok(points) = qdrant.search_ast_chunks("codepath", semantic_vector, 7).await {
        for point in points {
            if let Some(code) = point.get("content").and_then(|c| c.as_str()) {
                let file = point.get("file").and_then(|f| f.as_str()).unwrap_or("unknown_file");
                relevant_code_nodes.push(format!("--- AST Chunk Found inside [{}] ---\n{}\n---------------------------------", file, code));
            }
        }
    }

    if relevant_code_nodes.is_empty() {
        relevant_code_nodes.push("WARNING: No physical Qdrant vectors matched the semantic query payload. Ensure files are explicitly ingested into the Semantic Engine via the frontend interface first!".to_string());
    }

    ContextPackage {
        probable_modules,
        relevant_code_nodes,
    }
}
