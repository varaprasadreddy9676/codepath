/// CodePath MCP Server
///
/// A Model Context Protocol server that exposes CodePath's code analysis
/// capabilities as tools. Communicates via JSON-RPC 2.0 over stdio using
/// line-delimited JSON (one JSON message per line).
///
/// Tools:
///   - codepath_investigate: Query codebase for bugs, vulnerabilities, architecture issues
///   - codepath_ingest: Ingest a repository into the vector database
///   - codepath_pack: Pack a repo into a single LLM-friendly context document
///   - codepath_search: Direct vector search against Qdrant (no HTTP)
///   - codepath_health: Health check for the CodePath API
///   - codepath_job_status: Check async ingestion job status

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{self, BufRead, Write};

use ai_platform::embeddings;
use ai_platform::settings::Settings;
use ai_platform::storage::qdrant_adapter::QdrantAdapter;

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 Types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct JsonRpcMessage {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: Option<String>,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

// JSON-RPC error codes
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
#[allow(dead_code)]
const INTERNAL_ERROR: i64 = -32603;

// ---------------------------------------------------------------------------
// MCP Types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ToolDefinition {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn base_url() -> String {
    env::var("CODEPATH_API_URL").unwrap_or_else(|_| "http://localhost:3000".to_string())
}

fn success_response(id: Value, result: Value) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(result),
        error: None,
    }
}

fn error_response(id: Value, code: i64, message: &str) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: message.to_string(),
        }),
    }
}

fn tool_result(text: &str, is_error: bool) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error
    })
}

// ---------------------------------------------------------------------------
// Tool Definitions
// ---------------------------------------------------------------------------

fn get_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "codepath_investigate".to_string(),
            description: "Investigate code issues, bugs, security vulnerabilities, or architectural problems in an ingested codebase. Uses vector search to find relevant code. If an LLM API key is provided, CodePath runs its own analysis. If no key is provided, returns structured code evidence for YOU (the calling LLM) to analyze directly — no external API key needed.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The investigation query describing what to analyze"
                    },
                    "llm_api_key": {
                        "type": "string",
                        "description": "API key for the LLM provider"
                    },
                    "llm_api_url": {
                        "type": "string",
                        "description": "LLM API endpoint URL"
                    },
                    "llm_model": {
                        "type": "string",
                        "description": "LLM model name to use"
                    }
                },
                "required": ["text"]
            }),
        },
        ToolDefinition {
            name: "codepath_ingest".to_string(),
            description: "Ingest a repository into the CodePath vector database for analysis. Parses code into AST chunks and creates embeddings.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "repo_url": {
                        "type": "string",
                        "description": "Local path or URL of the repository to ingest"
                    },
                    "branch": {
                        "type": "string",
                        "description": "Git branch to ingest"
                    }
                },
                "required": ["repo_url"]
            }),
        },
        ToolDefinition {
            name: "codepath_pack".to_string(),
            description: "Pack a repository into a single LLM-friendly context document with directory tree, repo map, and code contents.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "repo_path": {
                        "type": "string",
                        "description": "Local filesystem path to the repository"
                    },
                    "style": {
                        "type": "string",
                        "description": "Output format: xml, markdown, or plain (default: xml)"
                    },
                    "compress": {
                        "type": "boolean",
                        "description": "Remove comments and docstrings to reduce size"
                    },
                    "include_patterns": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Glob patterns to include"
                    },
                    "exclude_patterns": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Glob patterns to exclude"
                    },
                    "include_git_diff": {
                        "type": "boolean",
                        "description": "Include git diff in output"
                    },
                    "include_git_log": {
                        "type": "boolean",
                        "description": "Include recent git log"
                    },
                    "show_line_numbers": {
                        "type": "boolean",
                        "description": "Show line numbers in code"
                    }
                },
                "required": ["repo_path"]
            }),
        },
        ToolDefinition {
            name: "codepath_search".to_string(),
            description: "Search the vector database for code chunks relevant to a query. Returns raw code snippets without LLM analysis.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query text"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default: 10)"
                    }
                },
                "required": ["query"]
            }),
        },
        ToolDefinition {
            name: "codepath_health".to_string(),
            description: "Check the health status of the CodePath API server and connected services.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDefinition {
            name: "codepath_job_status".to_string(),
            description: "Check the status of an async ingestion job.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "job_id": {
                        "type": "string",
                        "description": "The job ID returned by codepath_ingest"
                    }
                },
                "required": ["job_id"]
            }),
        },
    ]
}

// ---------------------------------------------------------------------------
// Protocol Handlers
// ---------------------------------------------------------------------------

fn handle_initialize(id: Value) -> JsonRpcResponse {
    eprintln!("[mcp] Handling initialize");
    success_response(
        id,
        json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {
                "tools": { "listChanged": false }
            },
            "serverInfo": {
                "name": "codepath-mcp",
                "version": "0.1.0"
            }
        }),
    )
}

fn handle_tools_list(id: Value) -> JsonRpcResponse {
    eprintln!("[mcp] Handling tools/list");
    let tools = get_tool_definitions();
    success_response(id, json!({ "tools": tools }))
}

async fn handle_tool_call(id: Value, params: &Value) -> JsonRpcResponse {
    let tool_name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    eprintln!("[mcp] Calling tool: {}", tool_name);

    let result = match tool_name {
        "codepath_investigate" => handle_investigate(&arguments).await,
        "codepath_ingest" => handle_ingest(&arguments).await,
        "codepath_pack" => handle_pack(&arguments).await,
        "codepath_search" => handle_search(&arguments).await,
        "codepath_health" => handle_health().await,
        "codepath_job_status" => handle_job_status(&arguments).await,
        _ => {
            return error_response(
                id,
                INVALID_PARAMS,
                &format!("Unknown tool: {}", tool_name),
            );
        }
    };

    match result {
        Ok(text) => success_response(id, tool_result(&text, false)),
        Err(e) => success_response(id, tool_result(&format!("Error: {}", e), true)),
    }
}

// ---------------------------------------------------------------------------
// Tool Handlers
// ---------------------------------------------------------------------------

async fn handle_investigate(args: &Value) -> Result<String, String> {
    let text = args
        .get("text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: text".to_string())?;

    let has_llm_key = args.get("llm_api_key").and_then(|v| v.as_str()).is_some_and(|k| !k.is_empty());

    if has_llm_key {
        // Full pipeline via CodePath API (uses provided LLM for analysis)
        let mut body = json!({ "text": text });
        if let Some(v) = args.get("llm_api_key").and_then(|v| v.as_str()) {
            body["llm_api_key"] = json!(v);
        }
        if let Some(v) = args.get("llm_api_url").and_then(|v| v.as_str()) {
            body["llm_api_url"] = json!(v);
        }
        if let Some(v) = args.get("llm_model").and_then(|v| v.as_str()) {
            body["llm_model"] = json!(v);
        }

        let url = format!("{}/api/v1/investigate", base_url());
        eprintln!("[mcp] POST {} (with LLM key)", url);

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = resp.status();
        let resp_text = resp
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("API returned {}: {}", status, resp_text));
        }

        if let Ok(parsed) = serde_json::from_str::<Value>(&resp_text) {
            if let Some(result) = parsed.get("result").and_then(|v| v.as_str()) {
                return Ok(result.to_string());
            }
        }
        Ok(resp_text)
    } else {
        // No API key — return structured evidence for the calling LLM to analyze
        eprintln!("[mcp] No LLM key provided. Returning raw evidence for caller's LLM to analyze.");

        let settings = Settings::load();
        let qdrant = QdrantAdapter::new(&settings.qdrant_url);
        let embedding = embeddings::embed_text(text).await;

        let results = qdrant
            .search_with_scores("codepath", embedding, 15, None)
            .await
            .map_err(|e| format!("Qdrant search failed: {}", e))?;

        if results.is_empty() {
            return Ok(format!(
                "INVESTIGATION QUERY: {}\n\nNo code chunks found in the vector database. \
                 Please ingest a repository first using codepath_ingest.", text
            ));
        }

        let mut output = format!(
            "INVESTIGATION QUERY: {}\n\n\
             The following code chunks were retrieved from the vector database via semantic search. \
             Analyze them to answer the investigation query. Focus on bugs, security issues, \
             error handling gaps, race conditions, and architectural problems.\n\n\
             ---\n\n",
            text
        );

        let mut total_chars = 0;
        let budget = 10000; // generous budget since the calling LLM has large context

        for (i, (payload, score)) in results.iter().enumerate() {
            let file_path = payload
                .get("file")
                .or_else(|| payload.get("file_path"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let content = payload
                .get("content")
                .or_else(|| payload.get("code"))
                .or_else(|| payload.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let language = payload
                .get("language")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if total_chars + content.len() > budget {
                break;
            }

            output.push_str(&format!(
                "### Chunk {} (relevance: {:.4})\n**File:** {}\n**Language:** {}\n```{}\n{}\n```\n\n",
                i + 1,
                score,
                file_path,
                language,
                language,
                content
            ));
            total_chars += content.len();
        }

        output.push_str("---\n\nPlease analyze the above code evidence and provide your findings.");
        Ok(output)
    }
}

async fn handle_ingest(args: &Value) -> Result<String, String> {
    let repo_url = args
        .get("repo_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: repo_url".to_string())?;

    let mut body = json!({ "repo_url": repo_url });
    if let Some(v) = args.get("branch").and_then(|v| v.as_str()) {
        body["branch"] = json!(v);
    }

    let url = format!("{}/api/v1/ingest", base_url());
    eprintln!("[mcp] POST {}", url);

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status();
    let resp_text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("API returned {}: {}", status, resp_text));
    }
    Ok(resp_text)
}

async fn handle_pack(args: &Value) -> Result<String, String> {
    let repo_path = args
        .get("repo_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: repo_path".to_string())?;

    let mut body = json!({ "repo_path": repo_path });

    // Forward all optional pack parameters
    for key in &[
        "style",
        "compress",
        "include_patterns",
        "exclude_patterns",
        "include_git_diff",
        "include_git_log",
        "show_line_numbers",
    ] {
        if let Some(v) = args.get(*key) {
            body[*key] = v.clone();
        }
    }

    let url = format!("{}/api/v1/pack", base_url());
    eprintln!("[mcp] POST {}", url);

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status();
    let resp_text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("API returned {}: {}", status, resp_text));
    }
    Ok(resp_text)
}

async fn handle_search(args: &Value) -> Result<String, String> {
    let query = args
        .get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: query".to_string())?;
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(10);

    eprintln!("[mcp] Direct Qdrant search: query={:?}, limit={}", query, limit);

    let settings = Settings::load();
    let qdrant = QdrantAdapter::new(&settings.qdrant_url);
    let embedding = embeddings::embed_text(query).await;

    let results = qdrant
        .search_with_scores("codepath", embedding, limit, None)
        .await
        .map_err(|e| format!("Qdrant search failed: {}", e))?;

    if results.is_empty() {
        return Ok("No results found.".to_string());
    }

    let mut output = String::new();
    for (i, (payload, score)) in results.iter().enumerate() {
        let file_path = payload
            .get("file")
            .or_else(|| payload.get("file_path"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let content = payload
            .get("content")
            .or_else(|| payload.get("code"))
            .or_else(|| payload.get("text"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let kind = payload
            .get("kind")
            .or_else(|| payload.get("chunk_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("chunk");

        output.push_str(&format!(
            "--- Result {} (score: {:.4}) ---\nFile: {}\nType: {}\n\n{}\n\n",
            i + 1,
            score,
            file_path,
            kind,
            content
        ));
    }
    Ok(output)
}

async fn handle_health() -> Result<String, String> {
    let url = format!("{}/api/health", base_url());
    eprintln!("[mcp] GET {}", url);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status();
    let resp_text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("Health check failed ({}): {}", status, resp_text));
    }
    Ok(resp_text)
}

async fn handle_job_status(args: &Value) -> Result<String, String> {
    let job_id = args
        .get("job_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing required parameter: job_id".to_string())?;

    let url = format!("{}/api/v1/jobs/{}", base_url(), job_id);
    eprintln!("[mcp] GET {}", url);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let status = resp.status();
    let resp_text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    if !status.is_success() {
        return Err(format!("API returned {}: {}", status, resp_text));
    }
    Ok(resp_text)
}

// ---------------------------------------------------------------------------
// Transport: Line-delimited JSON over stdio
// ---------------------------------------------------------------------------

/// Read a single JSON-RPC message from stdin (one JSON object per line).
/// Returns `None` on EOF.
fn read_message(stdin: &mut impl BufRead) -> Option<String> {
    let mut line = String::new();
    loop {
        line.clear();
        match stdin.read_line(&mut line) {
            Ok(0) => return None, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue; // skip blank lines
                }
                return Some(trimmed.to_string());
            }
            Err(e) => {
                eprintln!("[mcp] Error reading from stdin: {}", e);
                return None;
            }
        }
    }
}

/// Write a JSON-RPC response to stdout as a single line of JSON.
fn write_response(response: &JsonRpcResponse) {
    let body = match serde_json::to_string(response) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[mcp] Failed to serialize response: {}", e);
            return;
        }
    };

    let stdout = io::stdout();
    let mut writer = stdout.lock();

    if let Err(e) = writeln!(writer, "{}", body) {
        eprintln!("[mcp] Failed to write response: {}", e);
        return;
    }
    if let Err(e) = writer.flush() {
        eprintln!("[mcp] Failed to flush stdout: {}", e);
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    eprintln!("[mcp] CodePath MCP server starting...");
    eprintln!("[mcp] API base URL: {}", base_url());

    // Use a BufReader over raw stdin so buffered state persists across iterations.
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());

    loop {
        let raw = match read_message(&mut reader) {
            Some(msg) => msg,
            None => {
                eprintln!("[mcp] stdin closed, shutting down.");
                break;
            }
        };

        eprintln!("[mcp] Received: {}", raw);

        let msg: JsonRpcMessage = match serde_json::from_str(&raw) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[mcp] Parse error: {}", e);
                let resp = error_response(
                    Value::Null,
                    PARSE_ERROR,
                    &format!("Parse error: {}", e),
                );
                write_response(&resp);
                continue;
            }
        };

        if msg.jsonrpc != "2.0" {
            if let Some(id) = msg.id {
                let resp = error_response(id, INVALID_REQUEST, "Invalid jsonrpc version");
                write_response(&resp);
            }
            continue;
        }

        let method = match &msg.method {
            Some(m) => m.clone(),
            None => {
                if let Some(id) = msg.id {
                    let resp = error_response(id, INVALID_REQUEST, "Missing method");
                    write_response(&resp);
                }
                continue;
            }
        };

        // Notifications (no id) — handle silently, no response
        if msg.id.is_none() {
            eprintln!("[mcp] Notification: {}", method);
            continue;
        }

        let id = msg.id.unwrap();
        let params = msg.params.unwrap_or_else(|| json!({}));

        let response = match method.as_str() {
            "initialize" => handle_initialize(id),
            "tools/list" => handle_tools_list(id),
            "tools/call" => handle_tool_call(id, &params).await,
            _ => {
                eprintln!("[mcp] Unknown method: {}", method);
                error_response(
                    id,
                    METHOD_NOT_FOUND,
                    &format!("Method not found: {}", method),
                )
            }
        };

        write_response(&response);
    }

    eprintln!("[mcp] Server stopped.");
}
