# CodePath

A code intelligence engine built in Rust. Ingest any codebase into a vector database, then search, investigate, and pack it — all through a REST API or as an MCP tool provider for Claude Code, Cursor, Copilot, Windsurf, or Codex.

**Zero API keys required.** When used via MCP, your existing AI assistant IS the LLM — CodePath just handles the hard part: smart retrieval over massive codebases.

---

## What It Does

```
You ask: "Are there race conditions in payment processing?"

CodePath:
  1. Decomposes your query into sub-queries
  2. Generates 1024-dim embeddings (local, no GPU)
  3. Searches Qdrant vector DB across 2000+ code chunks
  4. Re-ranks by keyword overlap, definition boost, diversity
  5. Returns the exact code snippets that matter
  
Your LLM (Claude/GPT/Codex) analyzes them → actionable findings.
```

---

## Quick Start

### Prerequisites

- Rust 1.70+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- [Qdrant](https://qdrant.tech/documentation/quick-start/) running on `localhost:6333`

### Build & Run

```bash
git clone https://github.com/varaprasadreddy9676/codepath.git
cd codepath

# Build everything
cargo build --release

# Start the API server
cargo run --bin ai_platform
# → Listening on 127.0.0.1:3000
```

### Index a Codebase

```bash
curl -X POST http://localhost:3000/api/v1/ingest \
  -H "Content-Type: application/json" \
  -d '{"repo_url": "/path/to/your/project"}'
```

### Search It

```bash
curl -X POST http://localhost:3000/api/v1/investigate \
  -H "Content-Type: application/json" \
  -d '{"text": "Are there SQL injection vulnerabilities?"}'
```

That's it. No API keys, no config files, no cloud services.

---

## Two Ways to Use CodePath

### 1. MCP Tool Provider (Recommended)

Use CodePath as a tool inside Claude Code, Cursor, Copilot, Windsurf, or Codex. Your AI assistant calls CodePath tools directly — no API keys needed since the assistant itself is the LLM.

**Setup for VS Code / Copilot:**

The `.vscode/mcp.json` is already included. Just open the workspace.

**Setup for Claude Desktop:**

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "codepath": {
      "command": "/absolute/path/to/codepath/target/release/codepath-mcp",
      "env": {
        "CODEPATH_API_URL": "http://localhost:3000"
      }
    }
  }
}
```

**Setup for Cursor / Windsurf:**

Add to `.cursor/mcp.json` or `.windsurf/mcp.json` in your project:

```json
{
  "mcpServers": {
    "codepath": {
      "command": "/absolute/path/to/codepath/target/release/codepath-mcp"
    }
  }
}
```

#### MCP Tools

| Tool | What it does | Needs API key? |
|------|-------------|---------------|
| `codepath_investigate` | Find bugs, vulnerabilities, architecture issues | No |
| `codepath_ingest` | Index a repo into the vector DB | No |
| `codepath_pack` | Pack a repo into a single LLM-friendly document | No |
| `codepath_search` | Raw vector search for code chunks | No |
| `codepath_health` | Check if services are running | No |
| `codepath_job_status` | Check ingestion progress | No |

**How `codepath_investigate` works without an API key:**

When called without `llm_api_key`, CodePath returns structured code evidence (file paths, relevance scores, formatted code blocks) directly to the calling LLM. Claude/Copilot/Codex analyzes the evidence natively — often better than a free-tier Groq model would.

When called with `llm_api_key`, CodePath runs its own end-to-end analysis using the provided LLM.

### 2. REST API

For standalone use, CI/CD integration, or building your own tools.

#### `POST /api/v1/investigate`

Investigate code issues in an ingested codebase.

```bash
# Without LLM key — returns structured evidence
curl -X POST http://localhost:3000/api/v1/investigate \
  -H "Content-Type: application/json" \
  -d '{"text": "Are there error handling gaps in the API layer?"}'

# With LLM key — returns full LLM analysis
curl -X POST http://localhost:3000/api/v1/investigate \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Are there error handling gaps in the API layer?",
    "llm_api_key": "your-key",
    "llm_api_url": "https://api.groq.com/openai/v1/chat/completions",
    "llm_model": "llama-3.1-8b-instant"
  }'
```

#### `POST /api/v1/ingest`

Index a repository. Parses code into 60-line chunks with 10-line overlap, generates embeddings, stores in Qdrant.

```bash
curl -X POST http://localhost:3000/api/v1/ingest \
  -H "Content-Type: application/json" \
  -d '{"repo_url": "/path/to/repo", "branch": "main"}'

# Response: {"job_id": "uuid", "status": "processing"}
```

#### `POST /api/v1/pack`

Pack an entire repo into a single document for LLM context.

```bash
curl -X POST http://localhost:3000/api/v1/pack \
  -H "Content-Type: application/json" \
  -d '{
    "repo_path": "/path/to/repo",
    "style": "xml",
    "compress": true,
    "include_patterns": ["src/**/*.rs"],
    "exclude_patterns": ["**/tests/**"],
    "include_git_log": true,
    "include_git_diff": false,
    "show_line_numbers": false
  }'

# Response: {"content": "...", "total_tokens": 45000, "file_count": 120, "style": "xml"}
```

Output formats: `xml`, `markdown`, `plain`

#### `GET /api/v1/jobs/{job_id}`

Check async ingestion job status.

#### `GET /api/health`

Returns `"Platform Core is alive"`.

---

## How the Pipeline Works

```
User Query
    │
    ▼
┌─────────────┐
│ Interpreter  │  Classifies intent (technical_diagnosis, business_behavior,
│              │  visibility, state_transition, global_search) + extracts
│              │  entity IDs (BILL-1234, ORD-0999)
└──────┬──────┘
       │
       ▼
┌─────────────┐   7 strategies:
│  Context     │   1. Query decomposition ("auth AND payments" → 2 queries)
│  Resolver    │   2. Multi-vector search (each sub-query embedded separately)
│              │   3. Keyword boost (+0.10 for identifier matches)
│              │   4. Deduplication (by file + chunk_index)
│              │   5. Re-ranking (keyword overlap, definition boost, import penalty)
│              │   6. Adaptive budget (6K–12K chars based on query complexity)
│              │   7. Diversity selection (max 3 chunks per file)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Evidence    │  Code chunks from Qdrant + optional live DB state
│  Collector   │  (DB is skipped if no credentials configured)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Evaluator   │  Sends evidence to LLM (Groq, OpenRouter, OpenAI, Ollama)
│              │  Retries on 429 (rate limit) with retry-after header
│              │  Falls back to structural rules if no LLM key
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Composer    │  Formats diagnostic report with confidence score
└─────────────┘
```

---

## Smart Retrieval

CodePath doesn't just do a single vector search. It runs 7 retrieval strategies to find the most relevant code:

| Strategy | What it does |
|----------|-------------|
| Query decomposition | Splits complex queries on "and", "also", "plus", sentence boundaries |
| Multi-vector search | Each sub-query gets its own embedding and Qdrant search |
| Keyword boost | Searches for files matching extracted identifiers, boosts their scores |
| Deduplication | Removes duplicate chunks (same file + chunk index), keeps highest score |
| Re-ranking | Boosts definitions, penalizes imports, rewards keyword overlap |
| Adaptive budget | Simple queries get 6K chars, complex ones get up to 12K |
| Diversity | Caps chunks per file at 3 to avoid over-representing one file |

This is why CodePath scales to GB-sized codebases — it doesn't try to send everything to the LLM.

---

## Embeddings

CodePath uses local 1024-dimensional hash-based embeddings. No GPU, no API calls, fully deterministic.

```
Input: "async fn handle_payment(amount: f64)"
  → Tokenize on whitespace + punctuation
  → FNV-1a hash each token with 2 independent seeds
  → Map to bucket positions in 1024-dim vector
  → L2-normalize to unit length
  → Cosine similarity search in Qdrant
```

This captures keyword/identifier overlap — the primary signal for code search. Same text always produces the same vector.

---

## Supported Languages

Ingestion parses files with these extensions:

`.rs` `.js` `.ts` `.jsx` `.tsx` `.java` `.py` `.go` `.rb` `.vue` `.svelte` `.cs` `.kt`

Code is chunked into 60-line blocks with 10-line sliding overlap to preserve context across chunk boundaries.

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `QDRANT_URL` | `http://localhost:6333` | Qdrant vector database URL |
| `CODEPATH_API_URL` | `http://localhost:3000` | CodePath API URL (used by MCP server) |
| `TARGET_APP_DB_URL` | *(empty)* | Optional: connect to app's DB for live evidence |
| `OPENAI_API_KEY` | *(empty)* | Optional: LLM API key for server-side analysis |
| `LLM_API_URL` | `https://api.openai.com/v1/chat/completions` | LLM endpoint |
| `LLM_MODEL` | `gpt-4o` | LLM model name |
| `NEO4J_URL` | `bolt://localhost:7687` | Neo4j (future) |
| `JAEGER_URL` | `http://localhost:16686` | Jaeger tracing (future) |

---

## Why Use CodePath

**vs. grep / ripgrep:** CodePath understands semantic meaning. "authentication vulnerabilities" finds `auth.service.js`, `api.js`, and `authStore.js` — not just files containing the word "auth".

**vs. RAG chatbots:** CodePath's 7-strategy retrieval with re-ranking and diversity beats naive "embed query → top-k search". It handles GB-scale repos where simple RAG drowns in noise.

**vs. feeding whole repo to LLM:** Token limits. A 1GB codebase has millions of tokens. CodePath finds the 10 relevant chunks out of 2000+ and sends only those.

**vs. Claude Code / Codex alone:** They read files one at a time. CodePath pre-indexes the entire codebase into vectors, so a single query searches everything instantly. Use CodePath as their MCP tool for the best of both.

**Why MCP makes sense:** Your AI assistant already has a powerful LLM. It just needs a way to search large codebases efficiently. CodePath provides that through MCP — zero extra API keys, zero extra cost. The assistant calls `codepath_search` or `codepath_investigate`, gets structured code evidence, and analyzes it with its own built-in intelligence.

---

## Project Structure

```
src/
├── main.rs              # Axum REST API server (port 3000)
├── mcp_server.rs         # MCP server binary (stdio, JSON-RPC 2.0)
├── lib.rs               # Public modules
├── settings.rs          # Environment variable loading
├── embeddings.rs        # 1024-dim local hash embeddings
├── interpreter/         # Intent classification + entity extraction
├── context/             # 7-strategy smart retrieval from Qdrant
├── evidence/            # Code + optional DB evidence collection
├── evaluator/           # LLM analysis with retry logic
├── composer/            # Diagnostic report formatting
├── parsers/
│   ├── generic.rs       # 13-language code parser + chunker
│   └── java.rs          # Java-specific AST parser
├── storage/
│   ├── qdrant_adapter.rs # Vector DB (production)
│   ├── neo4j_adapter.rs  # Graph DB (stub)
│   └── tantivy_adapter.rs # Full-text search (stub)
├── context_engine/      # Repo packing (tree, git, symbols, formatting)
└── gatherers/           # DB + OpenTelemetry evidence adapters

tests/
├── context_engine_tests.rs  # 76 integration tests
└── pipeline_tests.rs        # 14 pipeline tests

ui/                      # React frontend (diagnostic console)
workers/java-parser/     # Java AST extractor (Maven)
mcp-config/              # Editor MCP configs (Claude, Cursor, Windsurf)
```

---

## Testing

```bash
cargo test
# 115 tests: 25 unit + 76 context engine + 14 pipeline
```

---

## Architecture

See [docs/architecture.md](docs/architecture.md) for the full technical deep-dive.

---

*Built for engineers who work with codebases too large to read, too complex to grep.*
