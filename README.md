# Generic Application Intelligence Platform

An open-source, Rust-based reasoning engine designed for codebase understanding, business behavior explanation, runtime diagnosis, and evidence-based root cause analysis.

## Overview
This platform goes beyond simple RAG (Retrieval-Augmented Generation) or LLM chat. It is a deterministic, five-stage pipeline designed to evaluate expected behavior against actual application state:

1. **Question Interpreter:** Normalizes the user question into an intent and extraction schema.
2. **Context Resolver:** Gathers semantic and structural code/config context.
3. **Evidence Collector:** Retrieves factual data (DB rows, logs, API responses).
4. **Constraint Evaluator:** Diagnoses the gap between expected rules and actual evidence.
5. **Explanation Composer:** Produces grounded, human-readable root cause answers.

## Getting Started

### Prerequisites
- Rust (1.70+)
- Cargo toolchain

### Installation
```bash
git clone https://github.com/codepath/ai_platform.git
cd ai_platform
cargo build
```

### Running the API
```bash
cargo run
```
The server will start on `127.0.0.1:3000`.

## Testing
We take software quality seriously. To run the integration and unit tests:
```bash
cargo test
```

## Documentation
- [Architecture Guide](docs/architecture.md)
- [Project Roadmap](ROADMAP.md)

## License
MIT
