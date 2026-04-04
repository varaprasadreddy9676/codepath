/// Context Engine: code2prompt + repomix + aider repo-map inspired features.
///
/// Provides:
/// - Token counting per file and total
/// - Directory tree visualization
/// - Git integration (logs, diffs, hot files by change frequency)
/// - Glob-based include/exclude filtering
/// - Repository map with key symbol extraction (aider-style)
/// - Context formatter packing repo into XML/Markdown/Plain for LLM consumption
/// - Code compression (signatures only, strip implementation bodies)

pub mod token_counter;
pub mod glob_filter;
pub mod tree_generator;
pub mod git_integration;
pub mod repo_map;
pub mod context_formatter;
