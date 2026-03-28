use std::env;
use dotenvy::dotenv;

pub struct Settings {
    pub target_app_db_url: String,
    pub qdrant_url: String,
    pub neo4j_url: String,
    pub jaeger_url: String,
    pub openai_api_key: String,
    pub llm_api_url: String,
}

impl Settings {
    pub fn load() -> Self {
        // Securely load dynamic cluster endpoints from the local .env manifest
        dotenv().ok(); 
        
        Self {
            target_app_db_url: env::var("TARGET_APP_DB_URL")
                .unwrap_or_else(|_| "postgresql://readonly_user:password@localhost/appdb".to_string()),
            qdrant_url: env::var("QDRANT_URL")
                .unwrap_or_else(|_| "http://localhost:6333".to_string()),
            neo4j_url: env::var("NEO4J_URL")
                .unwrap_or_else(|_| "bolt://localhost:7687".to_string()),
            jaeger_url: env::var("JAEGER_URL")
                .unwrap_or_else(|_| "http://localhost:16686".to_string()),
            openai_api_key: env::var("OPENAI_API_KEY").unwrap_or_default(),
            llm_api_url: env::var("LLM_API_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string()),
        }
    }
}
