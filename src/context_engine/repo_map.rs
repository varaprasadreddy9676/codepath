/// Repository map generator inspired by aider's repo-map concept.
/// Extracts key symbols (functions, structs, classes, interfaces) from source files
/// using regex-based pattern matching across multiple languages.
/// Produces a concise map showing file → key symbols with signatures.

use ignore::WalkBuilder;
use std::path::Path;

use super::glob_filter::GlobFilter;
use super::token_counter;

/// A symbol extracted from source code.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Symbol {
    pub name: String,
    pub kind: String,
    pub signature: String,
    pub line: usize,
}

/// A file entry in the repo map.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RepoMapEntry {
    pub file: String,
    pub language: String,
    pub tokens: usize,
    pub symbols: Vec<Symbol>,
}

/// Generate a repo map for the given repository path.
pub fn generate_repo_map(
    root: &str,
    include_patterns: &Option<Vec<String>>,
    exclude_patterns: &Option<Vec<String>>,
) -> Vec<RepoMapEntry> {
    let root_path = Path::new(root);
    let filter = GlobFilter::new(include_patterns, exclude_patterns);
    let mut entries = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .build();

    for result in walker.flatten() {
        let path = result.path();
        if !path.is_file() {
            continue;
        }

        let rel = match path.strip_prefix(root_path) {
            Ok(r) => r.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        if !filter.should_include(&rel) {
            continue;
        }

        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or_default();

        let language = match ext.as_str() {
            "rs" => "rust",
            "js" | "jsx" | "mjs" => "javascript",
            "ts" | "tsx" => "typescript",
            "java" => "java",
            "py" => "python",
            "go" => "go",
            "rb" => "ruby",
            "c" | "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" => "cpp",
            "cs" => "csharp",
            "swift" => "swift",
            "kt" | "kts" => "kotlin",
            _ => continue,
        };

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let tokens = token_counter::count_tokens(&content);
        let symbols = extract_symbols(&content, language);

        entries.push(RepoMapEntry {
            file: rel,
            language: language.to_string(),
            tokens,
            symbols,
        });
    }

    entries.sort_by(|a, b| b.tokens.cmp(&a.tokens));
    entries
}

/// Format repo map as concise text (aider-style).
pub fn format_repo_map(entries: &[RepoMapEntry]) -> String {
    let mut output = String::new();
    for entry in entries {
        output.push_str(&format!(
            "{}  ({}, {} tokens):\n",
            entry.file,
            entry.language,
            token_counter::format_token_count(entry.tokens)
        ));
        for sym in &entry.symbols {
            output.push_str(&format!("  {} {}: {}\n", sym.kind, sym.name, sym.signature));
        }
        if entry.symbols.is_empty() {
            output.push_str("  (no key symbols extracted)\n");
        }
        output.push('\n');
    }
    output
}

fn extract_symbols(content: &str, language: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }

        match language {
            "rust" => extract_rust_symbols(trimmed, i + 1, &mut symbols),
            "javascript" | "typescript" => extract_js_ts_symbols(trimmed, i + 1, &mut symbols),
            "java" | "kotlin" => extract_java_symbols(trimmed, i + 1, &mut symbols),
            "python" => extract_python_symbols(trimmed, i + 1, &mut symbols),
            "go" => extract_go_symbols(trimmed, i + 1, &mut symbols),
            "c" | "cpp" | "csharp" => extract_c_like_symbols(trimmed, i + 1, &mut symbols),
            _ => {}
        }
    }

    symbols
}

fn extract_rust_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    if line.starts_with("pub fn ") || line.starts_with("fn ") || line.starts_with("pub async fn ") || line.starts_with("async fn ") {
        if let Some(name) = extract_between(line, "fn ", "(") {
            symbols.push(Symbol { name, kind: "fn".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.starts_with("pub struct ") || line.starts_with("struct ") {
        if let Some(name) = extract_first_word_after(line, "struct ") {
            symbols.push(Symbol { name, kind: "struct".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.starts_with("pub enum ") || line.starts_with("enum ") {
        if let Some(name) = extract_first_word_after(line, "enum ") {
            symbols.push(Symbol { name, kind: "enum".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.starts_with("pub trait ") || line.starts_with("trait ") {
        if let Some(name) = extract_first_word_after(line, "trait ") {
            symbols.push(Symbol { name, kind: "trait".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.starts_with("impl ") {
        let name = line.trim_start_matches("impl ").split_whitespace().next().unwrap_or("").trim_end_matches('<').to_string();
        if !name.is_empty() {
            symbols.push(Symbol { name, kind: "impl".into(), signature: truncate_sig(line), line: line_num });
        }
    }
}

fn extract_js_ts_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    if line.starts_with("function ") || line.starts_with("export function ") || line.starts_with("async function ") || line.starts_with("export async function ") || line.starts_with("export default function ") {
        if let Some(name) = extract_between(line, "function ", "(") {
            symbols.push(Symbol { name, kind: "fn".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if (line.starts_with("const ") || line.starts_with("export const ") || line.starts_with("let "))
        && (line.contains("=>") || line.contains("= function"))
    {
        let word_pos = if line.starts_with("export ") { 2 } else { 1 };
        if let Some(name) = line.split_whitespace().nth(word_pos) {
            let name = name.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_').to_string();
            if !name.is_empty() {
                symbols.push(Symbol { name, kind: "fn".into(), signature: truncate_sig(line), line: line_num });
            }
        }
    } else if line.contains("class ") && (line.starts_with("class ") || line.starts_with("export class ") || line.starts_with("export default class ")) {
        if let Some(name) = extract_first_word_after(line, "class ") {
            symbols.push(Symbol { name, kind: "class".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.starts_with("interface ") || line.starts_with("export interface ") {
        if let Some(name) = extract_first_word_after(line, "interface ") {
            symbols.push(Symbol { name, kind: "interface".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.starts_with("type ") || line.starts_with("export type ") {
        if let Some(name) = extract_first_word_after(line, "type ") {
            let name = name.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_').to_string();
            if name != "of" && !name.is_empty() {
                symbols.push(Symbol { name, kind: "type".into(), signature: truncate_sig(line), line: line_num });
            }
        }
    }
}

fn extract_java_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    let has_mod = line.starts_with("public ") || line.starts_with("private ") || line.starts_with("protected ") || line.starts_with("abstract ") || line.starts_with("static ");
    if line.contains("class ") && !line.contains("//") && has_mod {
        if let Some(name) = extract_first_word_after(line, "class ") {
            symbols.push(Symbol { name, kind: "class".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.contains("interface ") && !line.contains("//") {
        if let Some(name) = extract_first_word_after(line, "interface ") {
            symbols.push(Symbol { name, kind: "interface".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.contains("enum ") && has_mod {
        if let Some(name) = extract_first_word_after(line, "enum ") {
            symbols.push(Symbol { name, kind: "enum".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if has_mod && line.contains('(') && !line.contains("new ") && !line.starts_with("if ") {
        if let Some(name) = extract_between(line, " ", "(") {
            let name = name.split_whitespace().last().unwrap_or("").to_string();
            if !name.is_empty() && name != "class" && name != "interface" {
                symbols.push(Symbol { name, kind: "method".into(), signature: truncate_sig(line), line: line_num });
            }
        }
    }
}

fn extract_python_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    if line.starts_with("def ") || line.starts_with("async def ") {
        if let Some(name) = extract_between(line, "def ", "(") {
            symbols.push(Symbol { name, kind: "fn".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.starts_with("class ") {
        let name = line.trim_start_matches("class ").split(|c: char| c == '(' || c == ':').next().unwrap_or("").trim().to_string();
        if !name.is_empty() {
            symbols.push(Symbol { name, kind: "class".into(), signature: truncate_sig(line), line: line_num });
        }
    }
}

fn extract_go_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    if line.starts_with("func ") {
        let name = if line.starts_with("func (") {
            extract_between(line, ") ", "(")
        } else {
            extract_between(line, "func ", "(")
        };
        if let Some(name) = name {
            symbols.push(Symbol { name, kind: "fn".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.starts_with("type ") && (line.contains("struct") || line.contains("interface")) {
        if let Some(name) = extract_first_word_after(line, "type ") {
            let kind = if line.contains("interface") { "interface" } else { "struct" };
            symbols.push(Symbol { name, kind: kind.into(), signature: truncate_sig(line), line: line_num });
        }
    }
}

fn extract_c_like_symbols(line: &str, line_num: usize, symbols: &mut Vec<Symbol>) {
    if line.starts_with("struct ") || line.starts_with("typedef struct") {
        if let Some(name) = extract_first_word_after(line, "struct ") {
            symbols.push(Symbol { name, kind: "struct".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.contains("class ") {
        if let Some(name) = extract_first_word_after(line, "class ") {
            symbols.push(Symbol { name, kind: "class".into(), signature: truncate_sig(line), line: line_num });
        }
    } else if line.contains("interface ") {
        if let Some(name) = extract_first_word_after(line, "interface ") {
            symbols.push(Symbol { name, kind: "interface".into(), signature: truncate_sig(line), line: line_num });
        }
    }
}

fn extract_between(text: &str, start: &str, end: &str) -> Option<String> {
    let start_idx = text.find(start)?;
    let after_start = &text[start_idx + start.len()..];
    let end_idx = after_start.find(end)?;
    let result = after_start[..end_idx].trim().to_string();
    if result.is_empty() { None } else { Some(result) }
}

fn extract_first_word_after(text: &str, marker: &str) -> Option<String> {
    let idx = text.find(marker)?;
    let after = &text[idx + marker.len()..];
    let word = after.split(|c: char| !c.is_alphanumeric() && c != '_').next()?.to_string();
    if word.is_empty() { None } else { Some(word) }
}

fn truncate_sig(line: &str) -> String {
    let s = line.trim();
    if s.len() > 120 { format!("{}...", &s[..117]) } else { s.to_string() }
}
