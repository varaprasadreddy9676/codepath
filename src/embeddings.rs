/// Lightweight text embedding utilities for vector search.
///
/// Supports two modes:
/// 1. Remote LLM embeddings (OpenAI-compatible API)
/// 2. Local deterministic bag-of-words hashing (no external dependency)
///
/// The local mode produces consistent vectors so that semantically similar
/// code chunks (sharing keywords, identifiers, patterns) land close together
/// in cosine-similarity space — good enough for code search without a GPU.

use reqwest::Client;
use serde_json::{json, Value};
use tracing::warn;

use crate::settings::Settings;

/// Vector dimension for local embeddings — must match Qdrant collection config.
pub const VECTOR_DIM: usize = 1024;

/// Generate an embedding vector for the given text.
/// Tries the configured LLM embeddings endpoint first; falls back to local hashing.
pub async fn embed_text(text: &str) -> Vec<f32> {
    let config = Settings::load();

    // Only attempt remote embeddings if a real API key is set (not "ollama" placeholder)
    if !config.openai_api_key.is_empty()
        && config.openai_api_key != "ollama"
        && config.llm_api_url.contains("http")
    {
        if let Some(vec) = try_remote_embedding(text, &config).await {
            return vec;
        }
    }

    // Deterministic local embedding
    local_embedding(text)
}

/// Try to get embeddings from an OpenAI-compatible API.
async fn try_remote_embedding(text: &str, config: &Settings) -> Option<Vec<f32>> {
    let url = config.llm_api_url.replace("/chat/completions", "/embeddings");
    let payload = json!({
        "model": config.llm_model,
        "input": text
    });

    let client = Client::new();
    match client
        .post(&url)
        .bearer_auth(&config.openai_api_key)
        .json(&payload)
        .send()
        .await
    {
        Ok(res) => {
            if let Ok(data) = res.json::<Value>().await {
                // OpenAI format: data[0].embedding
                if let Some(emb) = data
                    .get("data")
                    .and_then(|d| d.get(0))
                    .and_then(|d| d.get("embedding"))
                    .and_then(|e| e.as_array())
                {
                    let vec: Vec<f32> = emb.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                    if !vec.is_empty() {
                        return Some(vec);
                    }
                }
                // Ollama format: embedding directly
                if let Some(emb) = data.get("embedding").and_then(|e| e.as_array()) {
                    let vec: Vec<f32> = emb.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
                    if !vec.is_empty() {
                        return Some(vec);
                    }
                }
            }
        }
        Err(e) => warn!("Remote embedding API unreachable: {}", e),
    }
    None
}

/// Deterministic bag-of-words hash embedding.
///
/// Splits text into tokens (words, identifiers, operators), hashes each one,
/// and accumulates into a fixed-dimension vector. Normalises to unit length
/// so cosine similarity works correctly.
///
/// This is intentionally simple — it captures keyword/identifier overlap
/// which is the primary signal for code search.
pub fn local_embedding(text: &str) -> Vec<f32> {
    let mut vec = vec![0.0f32; VECTOR_DIM];

    if text.is_empty() {
        return vec;
    }

    // Tokenize: split on whitespace + common code punctuation
    for token in text.split(|c: char| c.is_whitespace() || "(){}[]<>;:,.=+-*/&|!@#$%^~`'\"\\".contains(c)) {
        let token = token.trim();
        if token.is_empty() || token.len() > 100 {
            continue;
        }

        let lower = token.to_lowercase();

        // Hash the token into multiple bucket positions (like a Bloom filter)
        // Using two independent hash functions for better distribution
        let h1 = simple_hash(lower.as_bytes(), 0x9e3779b9);
        let h2 = simple_hash(lower.as_bytes(), 0x517cc1b7);

        let idx1 = (h1 as usize) % VECTOR_DIM;
        let idx2 = (h2 as usize) % VECTOR_DIM;

        // Sign from h1's bit to get +/- values (random projection style)
        let sign = if (h1 >> 16) & 1 == 0 { 1.0 } else { -1.0 };

        vec[idx1] += sign;
        vec[idx2] += sign * 0.5; // Secondary bucket with lower weight
    }

    // L2-normalise to unit vector
    let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for v in &mut vec {
            *v /= norm;
        }
    }

    vec
}

/// Simple non-cryptographic hash (FNV-1a variant).
fn simple_hash(data: &[u8], seed: u32) -> u32 {
    let mut hash = seed;
    for &byte in data {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_embedding_deterministic() {
        let v1 = local_embedding("fn main() { println!(\"hello\"); }");
        let v2 = local_embedding("fn main() { println!(\"hello\"); }");
        assert_eq!(v1, v2, "Same text should produce identical vectors");
    }

    #[test]
    fn test_local_embedding_dimension() {
        let v = local_embedding("some code here");
        assert_eq!(v.len(), VECTOR_DIM);
    }

    #[test]
    fn test_local_embedding_unit_norm() {
        let v = local_embedding("pub fn process(data: &str) -> Result<()>");
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.01, "Vector should be unit-normalised, got {}", norm);
    }

    #[test]
    fn test_similar_texts_closer_than_different() {
        let v_payment = local_embedding("fn create_payment(amount: f64, lot_id: u64) -> Payment");
        let v_payment2 = local_embedding("fn record_payment(amount: f64, lot_id: u64) -> PaymentResult");
        let v_unrelated = local_embedding("struct DatabaseConfig { host: String, port: u16 }");

        let sim_similar = cosine_sim(&v_payment, &v_payment2);
        let sim_different = cosine_sim(&v_payment, &v_unrelated);

        assert!(
            sim_similar > sim_different,
            "Similar payment fns ({:.3}) should be closer than unrelated code ({:.3})",
            sim_similar, sim_different
        );
    }

    #[test]
    fn test_empty_text() {
        let v = local_embedding("");
        assert_eq!(v.len(), VECTOR_DIM);
        assert!(v.iter().all(|&x| x == 0.0), "Empty text should produce zero vector");
    }

    fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b).map(|(x, y)| x * y).sum()
    }
}
