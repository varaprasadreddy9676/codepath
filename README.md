# ⚡ App Diagnostics Reasoning Engine

Most AI coding assistants guess. This engine *proves*. 

Welcome to a deterministic, 5-stage diagnostic pipeline heavily optimized in **Rust**. This platform is built to parse vast codebases, fetch factual database evidence, and mathematically prove what your application *should* have done against what it *actually* did. 

## 🧠 Why This Exists
Debugging enterprise applications today means staring at distributed logs, tracing microservices, and guessing if a config flag caused a `NullPointerException`. 

We automate the absolute worst parts of root cause analysis. This platform routes every problem through a rigid, hallucination-free pipeline:

1. **Interpret:** Maps human questions (e.g., "Why didn't this discount apply?") to intent primitives.
2. **Contextualize:** Crawls the repo (using ASTs, Lexical, and Graph tools) to find the exact rules governing the feature.
3. **Collect Evidence:** Pulls the *live* data (Change logs, actual DB rows, active flags).
4. **Evaluate:** The diagnostic brain. It mathematically compares the expected code rules vs the actual evidence.
5. **Diagnose:** Synthesizes the exact break-point into a hyper-focused, human-friendly explanation.

## 🚀 Quick Start

Built on **Rust**, `axum`, and `tokio` for incredible performance and concurrency handling.

```bash
git clone https://github.com/varaprasadreddy9676/codepath.git
cd codepath

# Build the project
cargo build

# Spin up the reasoning API
cargo run
```
The server will boot up and bind to `127.0.0.1:3000`.

## 🧪 Bulletproof Testing
We treat software quality as a first-class citizen. Run the full unit and integration suite with a single command:

```bash
cargo test
```

## 🗺️ Where are we going?
We are rapidly building out multi-language syntax chunking (`tree-sitter`), JVM-specific deep abstractions (`JavaParser`), and Vector Database adapters (`Qdrant`). 

Check out the [Project Roadmap](ROADMAP.md) to see what's dropping next, and dive into the [Deep Architecture Docs](docs/architecture.md) to learn how the entire pipeline functions under the hood.

---
*Built for engineers who are tired of guessing.*
