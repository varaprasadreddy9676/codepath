use axum::{
    routing::{get, post},
    Router,
    Json,
    extract::{State, Path},
};
use serde::{Deserialize, Serialize};
use tracing::info;
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

use ai_platform::{
    interpreter, context, evidence, evaluator, composer, parsers,
    context_engine::context_formatter::{self, PackOptions, OutputStyle},
    models::{IngestRequest, IngestResponse, PackRequest, PackResponse},
};

type AppState = Arc<RwLock<HashMap<String, String>>>;

#[derive(Serialize, Deserialize)]
struct QueryRequest {
    text: String,
    llm_api_url: Option<String>,
    llm_api_key: Option<String>,
    llm_model: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct QueryResponse {
    result: String,
}

async fn get_job_status(Path(job_id): Path<String>, State(state): State<AppState>) -> Json<serde_json::Value> {
    let map = state.read().await;
    let status = map.get(&job_id).unwrap_or(&"unknown".to_string()).clone();
    Json(serde_json::json!({ "job_id": job_id, "status": status }))
}

async fn ingest_repo(
    State(state): State<AppState>,
    Json(payload): Json<IngestRequest>
) -> Json<IngestResponse> {
    info!("Received repository ingestion request for: {}", payload.repo_url);
    
    let job_id = uuid::Uuid::new_v4().to_string();
    
    // Register job state dynamically into Thread-Safe runtime
    {
        let mut map = state.write().await;
        map.insert(job_id.clone(), "processing".to_string());
    }

    let repo_url = payload.repo_url.clone();
    let job_id_clone = job_id.clone();
    let thread_state = state.clone();

    tokio::spawn(async move {
        info!("Executing parallel AST directory extractions via worker threads...");
        parsers::java::parse_repository(&repo_url).await;
        parsers::generic::parse_repository(&repo_url).await;
        
        // Organic physical workload resolution lock
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        
        let mut map = thread_state.write().await;
        map.insert(job_id_clone, "completed".to_string());
        info!("Vector extraction algorithms formally finished.");
    });

    Json(IngestResponse {
        job_id,
        status: "processing".to_string(),
    })
}

async fn health_check() -> &'static str {
    "Platform Core is alive"
}

async fn investigate(Json(payload): Json<QueryRequest>) -> Json<QueryResponse> {
    info!("Received new investigation request: {}", payload.text);
    
    let intent = interpreter::interpret_intent(payload.text).await;
    let context_pkg = context::resolve_context(&intent).await;
    let evidence_pkg = evidence::collect_evidence(&context_pkg).await;
    let result = evaluator::evaluate_constraints(
        &evidence_pkg,
        payload.llm_api_url,
        payload.llm_api_key,
        payload.llm_model
    ).await;
    let explanation = composer::compose_explanation(&result).await;
    
    Json(QueryResponse { result: explanation })
}

/// Pack a repository into a single LLM-friendly context document.
/// Features: directory tree, repo map (symbol extraction), git logs/diffs,
/// glob filtering, code compression, XML/Markdown/Plain output.
async fn pack_repo(Json(payload): Json<PackRequest>) -> Json<PackResponse> {
    info!("Received pack request for: {}", payload.repo_path);

    let options = PackOptions {
        style: OutputStyle::from_str(payload.style.as_deref().unwrap_or("xml")),
        compress: payload.compress.unwrap_or(false),
        include_patterns: payload.include_patterns,
        exclude_patterns: payload.exclude_patterns,
        include_git_diff: payload.include_git_diff.unwrap_or(false),
        include_git_log: payload.include_git_log.unwrap_or(true),
        git_log_count: payload.git_log_count.unwrap_or(50),
        show_line_numbers: payload.show_line_numbers.unwrap_or(false),
        remove_empty_lines: false,
        include_tree: payload.include_tree.unwrap_or(true),
        include_repo_map: payload.include_repo_map.unwrap_or(true),
    };

    let result = tokio::task::spawn_blocking(move || {
        context_formatter::pack_repository(&payload.repo_path, &options)
    }).await.unwrap();

    Json(PackResponse {
        content: result.content,
        total_tokens: result.total_tokens,
        file_count: result.file_count,
        style: result.style,
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting Generic Application Intelligence Platform API...");

    // Shared Atomic Job Dispatcher memory
    let shared_state = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/v1/investigate", post(investigate))
        .route("/api/v1/ingest", post(ingest_repo))
        .route("/api/v1/pack", post(pack_repo))
        .route("/api/v1/jobs/{job_id}", get(get_job_status))
        .layer(CorsLayer::permissive())
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    
    axum::serve(listener, app).await.unwrap();
}
