use axum::{
    routing::{get, post},
    Router,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use ai_platform::{
    interpreter, context, evidence, evaluator, composer, parsers, models::{IngestRequest, IngestResponse}
};

#[derive(Serialize, Deserialize)]
struct QueryRequest {
    text: String,
}

#[derive(Serialize, Deserialize)]
struct QueryResponse {
    result: String,
}

async fn ingest_repo(Json(payload): Json<IngestRequest>) -> Json<IngestResponse> {
    info!("Received repository ingestion request for: {}", payload.repo_url);
    
    let job_id = uuid::Uuid::new_v4().to_string();
    
    // Kick off async parse workers
    let repo_url = payload.repo_url.clone();
    tokio::spawn(async move {
        parsers::java::parse_repository(&repo_url).await;
        parsers::generic::parse_repository(&repo_url).await;
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
    
    // 1. Interpreter
    let intent = interpreter::interpret_intent(payload.text).await;
    
    // 2. Context Resolver
    let context_pkg = context::resolve_context(&intent).await;
    
    // 3. Evidence Collector
    let evidence_pkg = evidence::collect_evidence(&context_pkg).await;
    
    // 4. Constraint Evaluator
    let result = evaluator::evaluate_constraints(&evidence_pkg).await;
    
    // 5. Explanation Composer
    let explanation = composer::compose_explanation(&result).await;
    
    Json(QueryResponse { result: explanation })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting Generic Application Intelligence Platform API...");

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/v1/investigate", post(investigate))
        .route("/api/v1/ingest", post(ingest_repo));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    
    axum::serve(listener, app).await.unwrap();
}
