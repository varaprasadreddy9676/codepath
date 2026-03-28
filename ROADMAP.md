# Platform Roadmap

This roadmap outlines the path from initial scaffolding to a production-ready Generic Application Intelligence Platform.

## Phase 1: Core Engine Skeleton (Completed)
- [x] Rust workspace initialization (`tokio`, `axum`, `serde`).
- [x] Library and binary `src/` modularization.
- [x] Interfaces defined for Interpreter, Context, Evidence, Evaluator, and Composer.
- [x] E2E integration testing scaffold.

## Phase 2: Parsers and Extractor Integration (In Progress)
- [ ] Connect `JavaParser` for precise JVM AST extraction.
- [ ] Hook up `Tree-sitter` for multi-language syntax chunking.
- [ ] Establish webhook listeners for repository indexing synchronization.

## Phase 3: Knowledge Store Pluggability
- [ ] Implement `Tantivy` adapter for exact lexical query matching (stack traces, SQL IDs).
- [ ] Implement `Qdrant` adapter for semantic and hybrid vector retrieval.
- [ ] Implement Graph abstraction (compatible with Neo4j/Memgraph) for workflow and dependency traversal.

## Phase 4: Runtime Evidence Gatherers
- [ ] Read-only DB connection adapters for application state verification.
- [ ] OpenTelemetry log correlation ingestors.
- [ ] Advanced Change Data Capture (CDC) evidence gathering.

## Phase 5: Ecosystem and Tooling
- [ ] Fully-featured IDE plugin.
- [ ] Pre-built Knowledge Packs for popular frameworks (Spring Boot, Next.js).
