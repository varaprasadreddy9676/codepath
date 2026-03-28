use tracing::info;

pub async fn parse_repository(repo_url: &str) {
    info!("Initializing JavaParser AST extraction for repo: {}", repo_url);
    // STUB: Download repo and run JavaParser CLI/worker
}
