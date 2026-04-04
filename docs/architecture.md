# Architecture

## Overview

CodePath is a code intelligence engine that indexes codebases into a vector database and exposes them through a REST API and MCP (Model Context Protocol) server. It runs a 5-stage diagnostic pipeline and a 7-strategy smart retrieval system to find relevant code in repositories of any size.

Two binaries:
- `ai_platform` — Axum REST API on port 3000
- `codepath-mcp` — MCP server over stdio for Claude Code, Cursor, Copilot, Windsurf, Codex

## The 5-Stage Pipeline

```
User Query
    │
    ├─ "Are there race conditions in payment processing?"
    │
    ▼
┌──────────────────────────────────────────────────────┐
│ 1. INTERPRETER                                       │
│    Rule-based intent classification:                 │
│    • technical_diagnosis (error, exception, stack)    │
│    • business_behavior (why, discount, amount)       │
│    • visibility (can't see, where is)                │
│    • state_transition (draft, pending, status)       │
│    • global_search (everything else)                 │
│                                                      │
│    Entity extraction: BILL-1234, ORD-0999 via regex  │
│    Preserves original_text for embedding             │
└──────────┬───────────────────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────────────────┐
│ 2. CONTEXT RESOLVER (7-strategy smart retrieval)     │
│                                                      │
│    a) Query decomposition                            │
│       Splits on "and", "also", "plus", sentences     │
│                                                      │
│    b) Multi-vector search                            │
│       Each sub-query → 1024-dim embedding → Qdrant   │
│       5 chunks per sub-query (10 if single query)    │
│                                                      │
│    c) Keyword boost                                  │
│       Extract identifiers → scroll Qdrant for file   │
│       matches → +0.10 score bonus                    │
│                                                      │
│    d) Deduplication                                  │
│       By (file, chunk_index), keep highest score     │
│                                                      │
│    e) Re-ranking                                     │
│       Keyword overlap boost, definition boost,       │
│       import penalty                                 │
│                                                      │
│    f) Adaptive budget                                │
│       Simple queries: 6K chars                       │
│       Complex queries: up to 12K chars               │
│                                                      │
│    g) Diversity selection                            │
│       Max 3 chunks per file to avoid over-represent  │
└──────────┬───────────────────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────────────────┐
│ 3. EVIDENCE COLLECTOR                                │
│    Code evidence: chunks from context resolver       │
│    DB evidence: optional (skipped if no credentials) │
│    Checks TARGET_APP_DB_URL — skips if placeholder   │
└──────────┬───────────────────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────────────────┐
│ 4. EVALUATOR                                         │
│    With LLM key:                                     │
│      Sends code evidence (capped at 5500 chars)      │
│      to Groq/OpenRouter/OpenAI/Ollama                │
│      Retries on 429 with retry-after header          │
│      System prompt: "senior code auditor"            │
│      Max tokens: 1500                                │
│                                                      │
│    Without LLM key:                                  │
│      Falls back to structural rule evaluation        │
│      (DB flag checks, config mismatches)             │
└──────────┬───────────────────────────────────────────┘
           │
           ▼
┌──────────────────────────────────────────────────────┐
│ 5. COMPOSER                                          │
│    Formats diagnostic report with confidence score   │
│    < 0.9 → "moderate confidence, verify manually"    │
│    ≥ 0.9 → "strictly confirmed"                     │
└──────────────────────────────────────────────────────┘
```

## Embeddings

Local 1024-dimensional hash-based vectors. No GPU, no API calls, fully deterministic.

```
Token → FNV-1a hash (2 independent seeds)
     → 2 bucket positions in 1024-dim vector
     → Signed accumulation (+1.0, +0.5)
     → L2-normalized to unit length
     → Cosine similarity in Qdrant
```

Captures keyword and identifier overlap — the primary signal for code search. Remote embedding APIs (OpenAI, Ollama) supported as optional fallback.

## Ingestion

```
Repository path
    │
    ▼
File walker (ignore crate, respects .gitignore)
    │
    ├─ Filters: .rs .js .ts .jsx .tsx .java .py .go .rb .vue .svelte .cs .kt
    │
    ▼
Chunker (60 lines, 10-line overlap)
    │
    ▼
Embedder (1024-dim local hash per chunk)
    │
    ▼
Qdrant (collection: "codepath", Cosine distance)
    │
    ▼
Payload: { file, language, chunk_index, content }
```

Async via `POST /api/v1/ingest` — returns `job_id` for polling.

## MCP Server

The `codepath-mcp` binary implements MCP (Model Context Protocol) over stdio using line-delimited JSON-RPC 2.0.

```
Claude Code / Cursor / Copilot / Codex
    │
    │  stdio (JSON-RPC 2.0, line-delimited)
    │
    ▼
codepath-mcp
    │
    ├─ codepath_search ──→ Direct Qdrant query (no HTTP)
    ├─ codepath_investigate ──→ Qdrant search → structured evidence
    │                          (or HTTP API if LLM key provided)
    ├─ codepath_ingest ──→ HTTP → ai_platform API
    ├─ codepath_pack ──→ HTTP → ai_platform API
    ├─ codepath_health ──→ HTTP → ai_platform API
    └─ codepath_job_status ──→ HTTP → ai_platform API
```

**Keyless mode:** `codepath_investigate` without an API key returns structured code evidence (file paths, relevance scores, formatted code blocks) for the calling LLM to analyze directly. No external LLM cost.

**With API key:** Full pipeline — CodePath calls the specified LLM (Groq, OpenRouter, OpenAI, Ollama) and returns analyzed findings.

## Context Engine (Repo Packing)

`POST /api/v1/pack` generates a single LLM-friendly document from a repository:

| Component | What it does |
|-----------|-------------|
| Tree generator | Directory tree visualization |
| Repo map | Symbol extraction — functions, classes, types, constructors |
| Git integration | Recent commits, diffs, hot files |
| Glob filter | Include/exclude patterns for file selection |
| Code compression | Remove comments and docstrings |
| Token counter | Estimate token count for LLM budget planning |
| Formatter | Output as XML, Markdown, or Plain text |

## Module Map

```
src/
├── main.rs                    Axum API server (:3000)
├── mcp_server.rs              MCP binary (stdio JSON-RPC)
├── lib.rs                     Public module exports
├── settings.rs                Env var loading (Settings::load())
├── embeddings.rs              1024-dim local hash + remote fallback
│
├── interpreter/mod.rs         Intent classification + entity extraction
├── context/mod.rs             7-strategy smart retrieval from Qdrant
├── evidence/mod.rs            Code + optional DB evidence
├── evaluator/mod.rs           LLM analysis with retry + structural fallback
├── composer/mod.rs            Diagnostic report formatting
│
├── parsers/
│   ├── generic.rs             13-language chunker + embedder + ingester
│   └── java.rs                Java-specific AST parser
│
├── storage/
│   ├── qdrant_adapter.rs      Vector DB (production)
│   ├── neo4j_adapter.rs       Graph DB (stub)
│   └── tantivy_adapter.rs     Full-text search (stub)
│
├── gatherers/
│   ├── db_adapter.rs          Application DB state extraction
│   ├── opentelemetry_ingest.rs Trace log collection
│   └── cdc_adapter.rs         Change data capture (stub)
│
└── context_engine/
    ├── context_formatter.rs   Output generation (XML/MD/Plain)
    ├── git_integration.rs     Git log + diff extraction
    ├── glob_filter.rs         File pattern filtering
    ├── repo_map.rs            Symbol extraction
    ├── token_counter.rs       Token estimation
    └── tree_generator.rs      Directory tree
```

## Technology Stack

| Layer | Technology |
|-------|-----------|
| Core runtime | Rust, Tokio, Axum |
| Vector search | Qdrant (HTTP client via reqwest) |
| Embeddings | Local FNV-1a hash (1024-dim) |
| Code parsing | Generic file walker (ignore crate) + Java AST (workers/) |
| LLM providers | Groq, OpenRouter, OpenAI, Ollama (any OpenAI-compatible API) |
| MCP transport | JSON-RPC 2.0 over stdio |
| Frontend | React + Vite (ui/) |
| Full-text search | Tantivy (stub) |
| Graph DB | Neo4j (stub) |

## Data Flow: MCP Keyless Investigate

```
Claude Code asks: "Find SQL injection risks"
    │
    ▼
codepath-mcp receives tools/call
    │
    ▼
No llm_api_key → keyless mode
    │
    ▼
embed_text("Find SQL injection risks") → 1024-dim vector
    │
    ▼
Qdrant search (collection: "codepath", limit: 15)
    │
    ▼
Format results:
  ### Chunk 1 (relevance: 0.85)
  **File:** src/api/users.js
  **Language:** js
  ```js
  db.query(`SELECT * FROM users WHERE id = ${req.params.id}`)
  ```
    │
    ▼
Return to Claude Code → Claude analyzes the evidence
```

No API keys. No external LLM calls. The assistant IS the LLM.
