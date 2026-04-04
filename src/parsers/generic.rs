use tracing::{info, warn};
use ignore::WalkBuilder;
use crate::storage::qdrant_adapter::QdrantAdapter;
use crate::settings::Settings;
use crate::embeddings;
use serde_json::json;

pub async fn parse_repository(url: &str) {
    info!("Initializing explicit recursive filesystem crawl targeting root: {}", url);
    let config = Settings::load();
    let qdrant = QdrantAdapter::new(&config.qdrant_url);
    
    // Initialize Vector collection with our embedding dimension
    let _ = qdrant.provision_collection("codepath", embeddings::VECTOR_DIM as u64).await;
    
    let walker = WalkBuilder::new(url)
        .hidden(true)
        .git_ignore(true)
        .build();

    let mut nodes_ingested = 0;

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy();
                        if ext_str == "rs" || ext_str == "js" || ext_str == "ts" || ext_str == "java" || ext_str == "py"
                            || ext_str == "go" || ext_str == "rb" || ext_str == "jsx" || ext_str == "tsx"
                            || ext_str == "vue" || ext_str == "svelte" || ext_str == "cs" || ext_str == "kt" {
                            if let Ok(content) = tokio::fs::read_to_string(path).await {
                                let lines: Vec<&str> = content.lines().collect();
                                let chunk_size = 60;
                                let overlap = 10;
                                let mut chunks: Vec<(usize, String)> = Vec::new();

                                let mut start = 0;
                                let mut idx = 0;
                                while start < lines.len() {
                                    let end = (start + chunk_size).min(lines.len());
                                    let chunk_text = lines[start..end].join("\n");
                                    chunks.push((idx, chunk_text));
                                    idx += 1;
                                    start += chunk_size - overlap; // slide with overlap
                                }

                                for (i, chunk) in &chunks {
                                    if chunk.trim().is_empty() { continue; }
                                    
                                    let vector = embeddings::embed_text(chunk).await;
                                    
                                    let metadata = json!({
                                        "file": path.to_string_lossy().to_string(),
                                        "language": ext_str,
                                        "chunk_index": i,
                                        "content": chunk
                                    });
                                    
                                    if qdrant.ingest_ast_chunk("codepath", vector, metadata).await.is_ok() {
                                        nodes_ingested += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Err(err) => warn!("Walk crawler encountered physical fault: {}", err),
        }
    }
    info!("Generic parse worker fully finished. Successfully ingested {} AST chunks seamlessly into memory.", nodes_ingested);
}
