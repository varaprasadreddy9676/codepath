# Roadmap

## Phase 1: Core Engine (Done)
- [x] Rust workspace with `tokio`, `axum`, `serde`
- [x] Modular `src/` layout (lib.rs exports all pipeline stages)
- [x] 5-stage pipeline: Interpreter → Context → Evidence → Evaluator → Composer
- [x] 115 tests (25 unit + 76 context engine + 14 pipeline)

## Phase 2: Parsers & Ingestion (Done)
- [x] Generic multi-language parser (13 languages: rs, js, ts, jsx, tsx, java, py, go, rb, vue, svelte, cs, kt)
- [x] 60-line chunks with 10-line sliding overlap
- [x] Java AST extractor (Maven worker in `workers/java-parser/`)
- [x] Async ingestion via `POST /api/v1/ingest` with job tracking
- [ ] Tree-sitter grammar support beyond Rust (JS, TS, Java, Python grammars)
- [ ] Webhook listeners for repository indexing on push

## Phase 3: Vector Search & Smart Retrieval (Done)
- [x] Qdrant adapter — provision, ingest, search_with_scores, scroll_points
- [x] 1024-dim local hash embeddings (no GPU, deterministic, FNV-1a)
- [x] 7-strategy retrieval: query decomposition, multi-vector search, keyword boost, dedup, re-ranking, adaptive budget (6K–12K chars), diversity selection
- [x] Tested on real codebase (2400+ vectors, 5 diverse queries)
- [ ] Tantivy adapter for exact lexical matching (stack traces, SQL IDs) — stub exists
- [ ] Neo4j/Memgraph adapter for dependency graph traversal — stub exists
- [ ] Hybrid search (vector + lexical fusion)

## Phase 4: LLM Analysis & Evidence (Done)
- [x] LLM evaluator with configurable provider (OpenAI, Groq, OpenRouter, Ollama)
- [x] Retry logic with 429 rate-limit handling (retry-after header, 2 retries)
- [x] Evidence cap at 5500 chars for free-tier token budgets
- [x] DB evidence made optional — skips when no credentials configured
- [x] Structural rule fallback when no LLM key available
- [ ] OpenTelemetry log correlation — stub exists
- [ ] CDC (Change Data Capture) evidence gathering — stub exists

## Phase 5: Context Engine & Repo Packing (Done)
- [x] `POST /api/v1/pack` endpoint — pack entire repo into LLM-friendly document
- [x] 3 output formats: XML, Markdown, Plain
- [x] Directory tree generation
- [x] Repo map with symbol extraction (functions, classes, types)
- [x] Git log and diff integration
- [x] Glob-based include/exclude filtering
- [x] Code compression (comment/docstring removal)
- [x] Token counting for budget planning

## Phase 6: MCP Server (Done)
- [x] Rust MCP binary (`codepath-mcp`) — JSON-RPC 2.0 over stdio
- [x] 6 tools: investigate, ingest, pack, search, health, job_status
- [x] Zero API keys required — all tools work without external LLM
- [x] Keyless investigate mode: returns structured evidence for caller's LLM
- [x] Editor configs: VS Code, Claude Desktop, Cursor, Windsurf
- [x] Protocol: line-delimited JSON, proper error codes, notification handling

## Phase 7: Production Hardening (Next)
- [ ] Remote embeddings (OpenAI, Cohere, Voyage) with local fallback
- [ ] Incremental re-indexing (only changed files since last ingest)
- [ ] Collection management API (list, delete, stats)
- [ ] Authentication on REST API endpoints
- [ ] Rate limiting and request validation
- [ ] Dockerized deployment (API + Qdrant + MCP in one compose)
- [ ] CI/CD pipeline with automated test runs

## Phase 8: Ecosystem
- [ ] VS Code extension with UI (beyond MCP — inline diagnostics, code lens)
- [ ] Pre-built knowledge packs for popular frameworks (Spring Boot, Next.js)
- [ ] Multi-repo support (search across multiple indexed codebases)
- [ ] Real-time file watcher for auto-reindexing on save
- [ ] Neo4j dependency graph visualization
- [ ] Tantivy full-text search for exact string matching
