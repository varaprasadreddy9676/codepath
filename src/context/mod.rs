use crate::models::{InvestigationRequest, ContextPackage, ScoredChunk};
use tracing::info;
use crate::settings::Settings;
use crate::storage::qdrant_adapter::QdrantAdapter;
use crate::embeddings;
use serde_json::json;
use std::collections::{HashMap, HashSet};

/// Maximum total characters of code evidence to send downstream.
/// Adaptive: complex queries get more budget.
const BASE_CONTEXT_BUDGET: usize = 6000;
const MAX_CONTEXT_BUDGET: usize = 12000;

pub async fn resolve_context(request: &InvestigationRequest) -> ContextPackage {
    info!("Resolving context for intent: {}", request.intent);

    // Early exit for empty intent
    if request.intent.trim().is_empty() {
        return ContextPackage {
            probable_modules: vec![],
            relevant_code_nodes: vec!["ERROR: Investigation intent cannot be empty.".to_string()],
        };
    }

    let config = Settings::load();
    let qdrant = QdrantAdapter::new(&config.qdrant_url);

    // Use original text for embedding (not the classified intent label)
    let search_text = if request.original_text.is_empty() {
        &request.intent
    } else {
        &request.original_text
    };

    // --- Strategy 1: Decompose complex queries into sub-queries ---
    let sub_queries = decompose_query(search_text);
    info!("Query decomposed into {} sub-queries: {:?}", sub_queries.len(), sub_queries);

    // --- Strategy 2: Multi-vector search (one per sub-query) ---
    let mut all_chunks: Vec<ScoredChunk> = Vec::new();
    let chunks_per_query = if sub_queries.len() > 1 { 5 } else { 10 };

    for query in &sub_queries {
        let vector = embeddings::embed_text(query).await;
        if let Ok(results) = qdrant.search_with_scores("codepath", vector, chunks_per_query, None).await {
            for (payload, score) in results {
                if let Some(chunk) = parse_scored_chunk(&payload, score) {
                    all_chunks.push(chunk);
                }
            }
        }
    }

    // --- Strategy 3: Keyword-based file filtering ---
    // Extract key identifiers from the query (function names, file names, etc.)
    let keywords = extract_keywords(search_text);
    if !keywords.is_empty() {
        info!("Keyword boost: searching for {:?}", keywords);
        for keyword in &keywords {
            // Search for chunks from files whose path contains the keyword
            let filter = json!({
                "must": [{
                    "key": "file",
                    "match": { "value": keyword }
                }]
            });
            if let Ok(results) = qdrant.scroll_points("codepath", filter, 5).await {
                for payload in results {
                    // Give keyword-matched chunks a boost score
                    if let Some(mut chunk) = parse_scored_chunk(&payload, 0.15) {
                        chunk.score += 0.10; // Keyword match bonus
                        all_chunks.push(chunk);
                    }
                }
            }
        }
    }

    // --- Strategy 4: Deduplicate by file+chunk_index ---
    let mut seen = HashSet::new();
    let mut unique_chunks: Vec<ScoredChunk> = Vec::new();
    for chunk in all_chunks {
        let key = format!("{}:{}", chunk.file, chunk.chunk_index);
        if seen.insert(key) {
            unique_chunks.push(chunk);
        }
    }

    // --- Strategy 5: Re-rank by combined relevance ---
    rerank_chunks(&mut unique_chunks, search_text);
    unique_chunks.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    // --- Strategy 6: Adaptive context budget ---
    let budget = calculate_budget(search_text, unique_chunks.len());
    info!("Retrieval: {} unique chunks, budget {} chars", unique_chunks.len(), budget);

    // --- Strategy 7: Diversity — don't over-represent one file ---
    let selected = select_diverse_chunks(&unique_chunks, budget);
    info!("Selected {} chunks from {} unique files", selected.len(),
        selected.iter().map(|c| &c.file).collect::<HashSet<_>>().len());

    // Format into context package
    let mut relevant_code_nodes = Vec::new();
    let mut probable_modules = Vec::new();
    for chunk in &selected {
        let short_file = chunk.file.split('/').skip_while(|s| *s != "src" && *s != "backend" && *s != "frontend" && *s != "app" && *s != "lib")
            .collect::<Vec<_>>().join("/");
        let display_file = if short_file.is_empty() { &chunk.file } else { &short_file };
        relevant_code_nodes.push(format!(
            "--- {} [score: {:.3}] ---\n{}\n---",
            display_file, chunk.score, chunk.content
        ));
        if !probable_modules.contains(&chunk.file) {
            probable_modules.push(chunk.file.clone());
        }
    }

    if relevant_code_nodes.is_empty() {
        relevant_code_nodes.push("WARNING: No code chunks matched the query. Ensure the codebase is ingested first.".to_string());
    }

    ContextPackage {
        probable_modules,
        relevant_code_nodes,
    }
}

/// Decompose a complex query into focused sub-queries.
/// For simple queries, returns the original. For compound queries, splits them.
fn decompose_query(intent: &str) -> Vec<String> {
    let lower = intent.to_lowercase();
    let mut queries = Vec::new();

    // Split on conjunctions and question marks
    let separators = [" and ", " also ", " plus ", ". ", "? "];
    let mut parts: Vec<String> = vec![intent.to_string()];

    for sep in &separators {
        let mut new_parts = Vec::new();
        for part in &parts {
            for sub in part.split(sep) {
                let trimmed = sub.trim().to_string();
                if trimmed.len() > 10 {
                    new_parts.push(trimmed);
                }
            }
        }
        parts = new_parts;
    }

    // If we got meaningful splits, use them
    if parts.len() > 1 {
        queries.extend(parts);
    } else {
        queries.push(intent.to_string());
    }

    // Add a broader contextual query for complex topics
    if lower.contains("security") || lower.contains("vulnerab") || lower.contains("injection") {
        queries.push("SQL query user input validation sanitize".to_string());
    }
    if lower.contains("auth") || lower.contains("login") || lower.contains("jwt") || lower.contains("token") {
        queries.push("JWT token verify authenticate middleware session".to_string());
    }
    if lower.contains("payment") || lower.contains("transaction") || lower.contains("billing") {
        queries.push("payment amount transaction record create update".to_string());
    }
    if lower.contains("error") || lower.contains("exception") || lower.contains("fail") {
        queries.push("try catch error throw exception handle async".to_string());
    }
    if lower.contains("performance") || lower.contains("slow") || lower.contains("optimize") {
        queries.push("query index loop batch pagination limit cache".to_string());
    }

    // Dedup
    let mut seen = HashSet::new();
    queries.retain(|q| seen.insert(q.clone()));

    queries
}

/// Extract meaningful keywords (potential file names, function names, identifiers)
fn extract_keywords(intent: &str) -> Vec<String> {
    let mut keywords = Vec::new();

    // Look for camelCase or snake_case identifiers
    for word in intent.split_whitespace() {
        let clean: String = word.chars().filter(|c| c.is_alphanumeric() || *c == '_' || *c == '.').collect();
        if clean.is_empty() { continue; }

        // File-like patterns (contains dot with extension)
        if clean.contains('.') && clean.len() > 3 {
            keywords.push(clean);
            continue;
        }
        // snake_case identifiers
        if clean.contains('_') && clean.len() > 4 {
            keywords.push(clean);
            continue;
        }
        // CamelCase identifiers
        let upper_count = clean.chars().filter(|c| c.is_uppercase()).count();
        if upper_count >= 2 && clean.len() > 4 {
            keywords.push(clean);
        }
    }

    // Also extract domain terms that might be module/directory names
    let domain_terms = ["payment", "auth", "user", "admin", "lot", "member", "import",
        "export", "config", "middleware", "controller", "service", "repository", "route"];
    let lower = intent.to_lowercase();
    for term in &domain_terms {
        if lower.contains(term) {
            keywords.push(term.to_string());
        }
    }

    let mut seen = HashSet::new();
    keywords.retain(|k| seen.insert(k.clone()));
    keywords
}

/// Re-rank chunks with additional signals beyond raw vector similarity
fn rerank_chunks(chunks: &mut Vec<ScoredChunk>, intent: &str) {
    let intent_lower = intent.to_lowercase();
    let intent_words: Vec<&str> = intent_lower.split_whitespace().collect();

    for chunk in chunks.iter_mut() {
        let content_lower = chunk.content.to_lowercase();

        // Boost: keyword overlap between query and content
        let keyword_hits: usize = intent_words.iter()
            .filter(|w| w.len() > 3 && content_lower.contains(*w))
            .count();
        chunk.score += keyword_hits as f32 * 0.02;

        // Boost: function/class definitions are more valuable than plain code
        let def_patterns = ["function ", "async ", "const ", "class ", "def ", "fn ", "pub fn ", "impl "];
        let def_count = def_patterns.iter().filter(|p| content_lower.contains(*p)).count();
        chunk.score += def_count as f32 * 0.01;

        // Penalize: chunks that are mostly imports or config
        let import_lines = chunk.content.lines()
            .filter(|l| {
                let t = l.trim();
                t.starts_with("import ") || t.starts_with("require(") ||
                t.starts_with("const ") && t.contains("require(") ||
                t.starts_with("use ") || t.starts_with("from ")
            })
            .count();
        let total_lines = chunk.content.lines().count().max(1);
        if import_lines as f32 / total_lines as f32 > 0.6 {
            chunk.score *= 0.7; // Heavy import blocks are less useful
        }

        // Boost: short meaningful file paths (service, controller, middleware files)
        let valuable_paths = ["service", "controller", "middleware", "auth", "handler", "route", "model"];
        if valuable_paths.iter().any(|p| chunk.file.to_lowercase().contains(p)) {
            chunk.score += 0.02;
        }
    }
}

/// Adaptive budget: more context for complex multi-topic queries
fn calculate_budget(intent: &str, chunk_count: usize) -> usize {
    let word_count = intent.split_whitespace().count();
    let complexity = if word_count > 20 { 2 } else if word_count > 10 { 1 } else { 0 };

    let budget = BASE_CONTEXT_BUDGET + complexity * 2000;
    // Also scale up if we have many chunks to choose from
    let scaled = if chunk_count > 20 { budget + 2000 } else { budget };
    scaled.min(MAX_CONTEXT_BUDGET)
}

/// Select diverse chunks: limit per-file representation to avoid tunnel vision
fn select_diverse_chunks(chunks: &[ScoredChunk], budget: usize) -> Vec<ScoredChunk> {
    let mut selected = Vec::new();
    let mut file_counts: HashMap<String, usize> = HashMap::new();
    let mut used_chars = 0;
    let max_per_file = 3; // Don't take more than 3 chunks from one file

    for chunk in chunks {
        if used_chars + chunk.content.len() > budget { break; }

        let count = file_counts.entry(chunk.file.clone()).or_insert(0);
        if *count >= max_per_file { continue; }

        used_chars += chunk.content.len();
        *count += 1;
        selected.push(chunk.clone());
    }

    selected
}

/// Parse a Qdrant payload into a ScoredChunk
fn parse_scored_chunk(payload: &serde_json::Value, score: f32) -> Option<ScoredChunk> {
    let content = payload.get("content")?.as_str()?.to_string();
    let file = payload.get("file")?.as_str()?.to_string();
    let language = payload.get("language").and_then(|l| l.as_str()).unwrap_or("unknown").to_string();
    let chunk_index = payload.get("chunk_index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;

    Some(ScoredChunk { file, content, language, score, chunk_index })
}
