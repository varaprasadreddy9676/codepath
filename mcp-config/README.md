# CodePath MCP Server Configuration

Setup instructions for using CodePath as an MCP tool provider in different editors and AI assistants.

## Prerequisites

1. **Build the MCP server binary:**
   ```bash
   cargo build --release
   ```
   This produces `target/release/codepath-mcp`.

2. **Start the CodePath API server:**
   ```bash
   cargo run --bin ai_platform
   ```

3. **Ensure Qdrant is running** on `localhost:6333`:
   ```bash
   docker compose up qdrant -d
   ```

## Editor Setup

### VS Code / GitHub Copilot

The workspace already includes [`.vscode/mcp.json`](../.vscode/mcp.json) — no additional setup needed. Just open the workspace and the MCP server will be available to Copilot.

### Claude Desktop

1. Open your Claude Desktop config file:
   - **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Linux:** `~/.config/claude/claude_desktop_config.json`

2. Copy the contents of [`claude_desktop.json`](./claude_desktop.json) into it.

3. Update the `command` path to the absolute path of your `codepath-mcp` binary:
   ```json
   "command": "/Users/yourname/Documents/dev/codepath/target/release/codepath-mcp"
   ```

4. Restart Claude Desktop.

### Cursor

1. Create `.cursor/mcp.json` in your project root (or add to global Cursor settings).

2. Copy the contents of [`cursor.json`](./cursor.json) into it.

3. Update the `command` path to the absolute path of your `codepath-mcp` binary.

### Windsurf

Same format as Cursor. Add the config to `.windsurf/mcp.json` in your project root.

## Available Tools

| Tool | Description |
|------|-------------|
| `codepath_investigate` | Analyze code issues using LLM-powered investigation |
| `codepath_ingest` | Index a repository for code search and analysis |
| `codepath_pack` | Pack a repository into LLM-optimized context |
| `codepath_search` | Vector search across indexed code chunks |
| `codepath_health` | Check CodePath server health status |
| `codepath_job_status` | Check the status of an ingestion job |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CODEPATH_API_URL` | `http://localhost:3000` | Base URL of the CodePath API server |
