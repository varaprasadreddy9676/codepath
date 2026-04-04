/// Context formatter that packs repository content into LLM-friendly formats.
/// Inspired by repomix (XML/Markdown/Plain) and code2prompt (single prompt output).
/// Supports compression mode that strips implementation bodies, keeping only signatures.

use ignore::WalkBuilder;
use std::path::Path;
use tracing::info;

use super::glob_filter::GlobFilter;
use super::token_counter;
use super::tree_generator;
use super::repo_map;
use super::git_integration;
use crate::models::PackOutput;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputStyle {
    Xml,
    Markdown,
    Plain,
}

impl OutputStyle {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "xml" => Self::Xml,
            "markdown" | "md" => Self::Markdown,
            _ => Self::Plain,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackOptions {
    pub style: OutputStyle,
    pub compress: bool,
    pub include_patterns: Option<Vec<String>>,
    pub exclude_patterns: Option<Vec<String>>,
    pub include_git_diff: bool,
    pub include_git_log: bool,
    pub git_log_count: usize,
    pub show_line_numbers: bool,
    pub remove_empty_lines: bool,
    pub include_tree: bool,
    pub include_repo_map: bool,
}

impl Default for PackOptions {
    fn default() -> Self {
        Self {
            style: OutputStyle::Xml,
            compress: false,
            include_patterns: None,
            exclude_patterns: None,
            include_git_diff: false,
            include_git_log: false,
            git_log_count: 50,
            show_line_numbers: false,
            remove_empty_lines: false,
            include_tree: true,
            include_repo_map: true,
        }
    }
}

/// Pack an entire repository into a single LLM-friendly document.
pub fn pack_repository(repo_path: &str, options: &PackOptions) -> PackOutput {
    info!("Packing repository at {} with style {:?}", repo_path, options.style);
    let root_path = Path::new(repo_path);
    let filter = GlobFilter::new(&options.include_patterns, &options.exclude_patterns);

    let dir_tree = if options.include_tree {
        tree_generator::generate_tree(repo_path, &options.include_patterns, &options.exclude_patterns)
    } else {
        String::new()
    };

    let repo_map_text = if options.include_repo_map {
        let entries = repo_map::generate_repo_map(repo_path, &options.include_patterns, &options.exclude_patterns);
        repo_map::format_repo_map(&entries)
    } else {
        String::new()
    };

    let mut files: Vec<(String, String, usize)> = Vec::new();
    let mut file_count = 0usize;

    let walker = WalkBuilder::new(repo_path)
        .hidden(true)
        .git_ignore(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() { continue; }

        let rel = match path.strip_prefix(root_path) {
            Ok(r) => r.to_string_lossy().to_string(),
            Err(_) => continue,
        };

        if !filter.should_include(&rel) || is_binary_ext(&rel) {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let processed = if options.compress {
            compress_content(&content, &rel)
        } else if options.remove_empty_lines {
            content.lines().filter(|l| !l.trim().is_empty()).collect::<Vec<_>>().join("\n")
        } else {
            content
        };

        let final_content = if options.show_line_numbers {
            processed.lines().enumerate()
                .map(|(i, l)| format!("{:>4} | {}", i + 1, l))
                .collect::<Vec<_>>().join("\n")
        } else {
            processed
        };

        let tokens = token_counter::count_tokens(&final_content);
        file_count += 1;
        files.push((rel, final_content, tokens));
    }

    // Git integration
    let git_section = build_git_section(repo_path, options);

    let content = match options.style {
        OutputStyle::Xml => format_xml(&dir_tree, &repo_map_text, &files, &git_section),
        OutputStyle::Markdown => format_markdown(&dir_tree, &repo_map_text, &files, &git_section),
        OutputStyle::Plain => format_plain(&dir_tree, &repo_map_text, &files, &git_section),
    };

    let total_tokens = token_counter::count_tokens(&content);

    PackOutput {
        content,
        total_tokens,
        file_count,
        style: format!("{:?}", options.style).to_lowercase(),
    }
}

fn build_git_section(repo_path: &str, options: &PackOptions) -> String {
    if !options.include_git_diff && !options.include_git_log {
        return String::new();
    }
    let mut git_text = String::new();
    if let Some(summary) = git_integration::gather_git_summary(repo_path, options.git_log_count) {
        if options.include_git_log && !summary.recent_commits.is_empty() {
            git_text.push_str("Recent Commits:\n");
            for commit in &summary.recent_commits {
                git_text.push_str(&format!("  {} ({}) {}\n", commit.hash, commit.date, commit.message));
                for f in &commit.files {
                    git_text.push_str(&format!("    - {}\n", f));
                }
            }
            git_text.push('\n');
            if !summary.hot_files.is_empty() {
                git_text.push_str("Hot Files (most frequently changed):\n");
                for (file, count) in &summary.hot_files {
                    git_text.push_str(&format!("  {:>3}x  {}\n", count, file));
                }
                git_text.push('\n');
            }
        }
        if options.include_git_diff && !summary.diff_stat.is_empty() {
            git_text.push_str(&summary.diff_stat);
            git_text.push('\n');
        }
    }
    git_text
}

fn format_xml(tree: &str, repo_map: &str, files: &[(String, String, usize)], git_section: &str) -> String {
    let mut out = String::new();
    out.push_str("<repository_context>\n");
    out.push_str("<file_summary>\nPacked representation of the repository for LLM context.\n</file_summary>\n\n");

    if !tree.is_empty() {
        out.push_str("<directory_structure>\n");
        out.push_str(tree);
        out.push_str("</directory_structure>\n\n");
    }
    if !repo_map.is_empty() {
        out.push_str("<repository_map>\n");
        out.push_str(repo_map);
        out.push_str("</repository_map>\n\n");
    }
    if !git_section.is_empty() {
        out.push_str("<git_context>\n");
        out.push_str(git_section);
        out.push_str("</git_context>\n\n");
    }
    out.push_str("<files>\n");
    for (path, content, tokens) in files {
        out.push_str(&format!("<file path=\"{}\" tokens=\"{}\">\n{}\n</file>\n\n", path, tokens, content));
    }
    out.push_str("</files>\n</repository_context>\n");
    out
}

fn format_markdown(tree: &str, repo_map: &str, files: &[(String, String, usize)], git_section: &str) -> String {
    let mut out = String::from("# Repository Context\n\n");
    if !tree.is_empty() {
        out.push_str("## Directory Structure\n\n```\n");
        out.push_str(tree);
        out.push_str("```\n\n");
    }
    if !repo_map.is_empty() {
        out.push_str("## Repository Map\n\n```\n");
        out.push_str(repo_map);
        out.push_str("```\n\n");
    }
    if !git_section.is_empty() {
        out.push_str("## Git Context\n\n```\n");
        out.push_str(git_section);
        out.push_str("```\n\n");
    }
    out.push_str("## Files\n\n");
    for (path, content, tokens) in files {
        let ext = path.rsplit('.').next().unwrap_or("");
        out.push_str(&format!("### {} ({} tokens)\n\n```{}\n{}\n```\n\n",
            path, token_counter::format_token_count(*tokens), ext, content));
    }
    out
}

fn format_plain(tree: &str, repo_map: &str, files: &[(String, String, usize)], git_section: &str) -> String {
    let sep = "================================================================\n";
    let mut out = String::new();
    out.push_str(sep); out.push_str("Repository Context\n"); out.push_str(sep); out.push('\n');
    if !tree.is_empty() {
        out.push_str(sep); out.push_str("Directory Structure\n"); out.push_str(sep);
        out.push_str(tree); out.push('\n');
    }
    if !repo_map.is_empty() {
        out.push_str(sep); out.push_str("Repository Map\n"); out.push_str(sep);
        out.push_str(repo_map); out.push('\n');
    }
    if !git_section.is_empty() {
        out.push_str(sep); out.push_str("Git Context\n"); out.push_str(sep);
        out.push_str(git_section); out.push('\n');
    }
    out.push_str(sep); out.push_str("Files\n"); out.push_str(sep); out.push('\n');
    for (path, content, tokens) in files {
        out.push_str(&format!("================\nFile: {} ({} tokens)\n================\n{}\n\n",
            path, token_counter::format_token_count(*tokens), content));
    }
    out
}

/// Compress source code: keep signatures, imports, types; strip function bodies.
fn compress_content(content: &str, file_path: &str) -> String {
    let ext = file_path.rsplit('.').next().unwrap_or("");
    let lines: Vec<&str> = content.lines().collect();
    let mut output = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut skip_body = false;

    for line in &lines {
        let trimmed = line.trim();

        if is_import_line(trimmed, ext) {
            output.push(*line);
            continue;
        }

        if is_definition_line(trimmed, ext) {
            if skip_body && brace_depth > 0 {
                output.push("⋮----");
            }
            skip_body = false;
            output.push(*line);
            if is_function_start(trimmed, ext) {
                skip_body = true;
            }
            continue;
        }

        let opens = trimmed.matches('{').count() as i32;
        let closes = trimmed.matches('}').count() as i32;
        brace_depth += opens - closes;

        if skip_body && brace_depth > 0 { continue; }
        if skip_body && brace_depth <= 0 {
            skip_body = false;
            if trimmed == "}" { continue; }
        }

        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') || trimmed.starts_with('@') {
            output.push(*line);
            continue;
        }

        if !skip_body { output.push(*line); }
    }

    output.join("\n")
}

fn is_import_line(line: &str, ext: &str) -> bool {
    match ext {
        "rs" => line.starts_with("use ") || line.starts_with("pub use ") || line.starts_with("mod ") || line.starts_with("pub mod "),
        "js" | "jsx" | "ts" | "tsx" | "mjs" => line.starts_with("import ") || (line.starts_with("export ") && line.contains(" from ")),
        "java" | "kt" => line.starts_with("import ") || line.starts_with("package "),
        "py" => line.starts_with("import ") || line.starts_with("from "),
        "go" => line.starts_with("import ") || line.starts_with("package "),
        "c" | "cpp" | "h" | "hpp" => line.starts_with("#include") || line.starts_with("#define"),
        "cs" => line.starts_with("using ") || line.starts_with("namespace "),
        _ => false,
    }
}

fn is_definition_line(line: &str, ext: &str) -> bool {
    match ext {
        "rs" => line.starts_with("pub fn ") || line.starts_with("fn ") || line.starts_with("pub async fn ") || line.starts_with("async fn ")
            || line.starts_with("pub struct ") || line.starts_with("struct ") || line.starts_with("pub enum ") || line.starts_with("enum ")
            || line.starts_with("pub trait ") || line.starts_with("trait ") || line.starts_with("impl "),
        "js" | "jsx" | "ts" | "tsx" | "mjs" => line.starts_with("function ") || line.starts_with("export function ")
            || line.starts_with("export default function ") || line.starts_with("async function ") || line.starts_with("export async function ")
            || line.starts_with("class ") || line.starts_with("export class ") || line.starts_with("export default class ")
            || line.starts_with("interface ") || line.starts_with("export interface ")
            || ((line.starts_with("const ") || line.starts_with("export const ")) && (line.contains("=>") || line.contains("= function"))),
        "java" | "kt" => (line.starts_with("public ") || line.starts_with("private ") || line.starts_with("protected ")
            || line.starts_with("class ") || line.starts_with("interface ") || line.starts_with("enum "))
            && (line.contains('(') || line.contains("class ") || line.contains("interface ") || line.contains("enum ")),
        "py" => line.starts_with("def ") || line.starts_with("async def ") || line.starts_with("class "),
        "go" => line.starts_with("func ") || (line.starts_with("type ") && (line.contains("struct") || line.contains("interface"))),
        _ => false,
    }
}

fn is_function_start(line: &str, ext: &str) -> bool {
    match ext {
        "rs" => line.contains("fn ") && (line.contains('{') || !line.contains(';')),
        "js" | "jsx" | "ts" | "tsx" | "mjs" => (line.contains("function ") || line.contains("=>")) && line.contains('{'),
        "java" | "kt" => line.contains('(') && line.contains('{') && !line.contains("class ") && !line.contains("interface "),
        "py" => (line.starts_with("def ") || line.starts_with("async def ")) && line.ends_with(':'),
        "go" => line.starts_with("func ") && line.contains('{'),
        _ => false,
    }
}

fn is_binary_ext(path: &str) -> bool {
    let binary = ["png","jpg","jpeg","gif","bmp","ico","webp","svg","mp3","mp4","avi","mov","wav",
        "zip","tar","gz","bz2","7z","rar","exe","dll","so","dylib","o","a",
        "woff","woff2","ttf","otf","eot","pdf","doc","docx","xls","xlsx","class","pyc","wasm","lock"];
    path.rsplit('.').next().map(|e| binary.contains(&e.to_lowercase().as_str())).unwrap_or(false)
}
