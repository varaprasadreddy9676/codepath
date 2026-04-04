use ai_platform::context_engine::{
    context_formatter::{self, OutputStyle, PackOptions},
    glob_filter::GlobFilter,
    repo_map,
    token_counter,
    tree_generator,
    git_integration,
};
use std::fs;
use tempfile::TempDir;

// ============================================================
// Helper: create a temp repo with realistic file structure
// ============================================================

fn create_test_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create directory structure
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("src/models")).unwrap();
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();

    // Rust files
    fs::write(
        root.join("src/main.rs"),
        r#"use std::collections::HashMap;

pub fn main() {
    println!("Hello, world!");
    let x = 42;
    let y = x + 1;
}

pub struct AppConfig {
    pub name: String,
    pub port: u16,
}

pub enum Status {
    Active,
    Inactive,
    Pending,
}

pub trait Handler {
    fn handle(&self) -> String;
}

impl AppConfig {
    pub fn new(name: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            port,
        }
    }
}

fn helper_function(x: i32) -> i32 {
    x * 2
}

pub async fn async_handler() {
    println!("async!");
}
"#,
    )
    .unwrap();

    fs::write(
        root.join("src/models/mod.rs"),
        r#"use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
}

pub fn create_user(id: u64, name: &str) -> User {
    User { id, name: name.to_string() }
}
"#,
    )
    .unwrap();

    // JavaScript file
    fs::write(
        root.join("src/app.js"),
        r#"import { useState } from 'react';
import axios from 'axios';

export function fetchData(url) {
    return axios.get(url);
}

export class ApiClient {
    constructor(baseUrl) {
        this.baseUrl = baseUrl;
    }

    async get(path) {
        return fetch(this.baseUrl + path);
    }
}

const processItems = (items) => {
    return items.filter(i => i.active);
};

export default function App() {
    return <div>Hello</div>;
}
"#,
    )
    .unwrap();

    // Python file
    fs::write(
        root.join("src/utils.py"),
        r#"import os
from pathlib import Path

def process_data(data):
    return [item for item in data if item]

class DataProcessor:
    def __init__(self, config):
        self.config = config

    def run(self):
        return self.config

async def fetch_remote(url):
    pass
"#,
    )
    .unwrap();

    // Java file
    fs::write(
        root.join("src/Service.java"),
        r#"package com.example;

import java.util.List;

public class UserService {
    private final UserRepository repo;

    public UserService(UserRepository repo) {
        this.repo = repo;
    }

    public List<User> findAll() {
        return repo.findAll();
    }
}

public interface UserRepository {
    List<User> findAll();
}

public enum Role {
    ADMIN,
    USER,
    GUEST
}
"#,
    )
    .unwrap();

    // Go file
    fs::write(
        root.join("src/handler.go"),
        r#"package main

import "fmt"

func main() {
    fmt.Println("hello")
}

type Config struct {
    Name string
    Port int
}

type Handler interface {
    Handle() error
}

func (c *Config) Validate() error {
    return nil
}
"#,
    )
    .unwrap();

    // TypeScript file
    fs::write(
        root.join("src/types.ts"),
        r#"import { z } from 'zod';

export interface UserDTO {
    id: number;
    name: string;
}

export type CreateUserInput = Omit<UserDTO, 'id'>;

export class UserService {
    constructor(private repo: UserRepository) {}

    async getUser(id: number): Promise<UserDTO> {
        return this.repo.find(id);
    }
}

export function validateUser(user: UserDTO): boolean {
    return user.id > 0 && user.name.length > 0;
}

export const fetchUsers = async () => {
    return [];
};
"#,
    )
    .unwrap();

    // Markdown doc
    fs::write(root.join("docs/README.md"), "# Test Project\n\nA test project.\n").unwrap();

    // A test file
    fs::write(
        root.join("tests/test_main.rs"),
        r#"#[test]
fn it_works() {
    assert_eq!(2 + 2, 4);
}
"#,
    )
    .unwrap();

    // Root config file (non-code)
    fs::write(root.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

    // Binary-extension file (should be skipped by context_formatter)
    fs::write(root.join("image.png"), "fake binary data").unwrap();

    dir
}

fn create_git_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Initialize a git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(root)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(root)
        .output()
        .unwrap();

    // Create files and commits
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("README.md"), "# Test\n").unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(root)
        .output()
        .unwrap();

    // Second commit
    fs::write(root.join("src/main.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
    fs::write(root.join("src/lib.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["commit", "-m", "Add lib and update main"])
        .current_dir(root)
        .output()
        .unwrap();

    // Third commit - modify main again (makes it a "hot file")
    fs::write(
        root.join("src/main.rs"),
        "use crate::add;\nfn main() {\n    let x = add(1, 2);\n    println!(\"{}\", x);\n}\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(["commit", "-m", "Refactor main to use lib"])
        .current_dir(root)
        .output()
        .unwrap();

    dir
}

// ============================================================
// TOKEN COUNTER TESTS
// ============================================================

#[test]
fn token_counter_empty_string() {
    assert_eq!(token_counter::count_tokens(""), 0);
}

#[test]
fn token_counter_single_char() {
    assert_eq!(token_counter::count_tokens("a"), 1);
}

#[test]
fn token_counter_exact_multiple_of_four() {
    // 8 chars → 2 tokens
    assert_eq!(token_counter::count_tokens("abcdefgh"), 2);
}

#[test]
fn token_counter_off_by_one() {
    // 9 chars → ceil(9/4) = 3
    assert_eq!(token_counter::count_tokens("abcdefghi"), 3);
    // 7 chars → ceil(7/4) = 2 — (7+3)/4 = 2
    assert_eq!(token_counter::count_tokens("abcdefg"), 2);
}

#[test]
fn token_counter_realistic_code() {
    let code = r#"pub fn main() {
    println!("Hello, world!");
}"#;
    let tokens = token_counter::count_tokens(code);
    assert!(tokens > 0);
    // ~50 chars → ~12-13 tokens
    assert!(tokens >= 10 && tokens <= 20);
}

#[test]
fn token_counter_unicode() {
    // Unicode chars are multi-byte; count_tokens uses byte length
    let text = "日本語のテスト";
    let tokens = token_counter::count_tokens(text);
    assert!(tokens > 0);
}

#[test]
fn token_counter_whitespace_only() {
    assert_eq!(token_counter::count_tokens("    "), 1); // 4 chars
    assert_eq!(token_counter::count_tokens("  "), 1);   // 2 chars → (2+3)/4 = 1
}

#[test]
fn token_counter_large_content() {
    let large = "x".repeat(1_000_000);
    assert_eq!(token_counter::count_tokens(&large), 250_000);
}

#[test]
fn format_token_count_units() {
    assert_eq!(token_counter::format_token_count(0), "0");
    assert_eq!(token_counter::format_token_count(1), "1");
    assert_eq!(token_counter::format_token_count(999), "999");
    assert_eq!(token_counter::format_token_count(1000), "1.0k");
    assert_eq!(token_counter::format_token_count(1500), "1.5k");
    assert_eq!(token_counter::format_token_count(10_000), "10.0k");
    assert_eq!(token_counter::format_token_count(999_999), "1000.0k");
    assert_eq!(token_counter::format_token_count(1_000_000), "1.0M");
    assert_eq!(token_counter::format_token_count(1_500_000), "1.5M");
    assert_eq!(token_counter::format_token_count(10_000_000), "10.0M");
}

// ============================================================
// GLOB FILTER TESTS
// ============================================================

#[test]
fn glob_filter_no_patterns_includes_everything() {
    let filter = GlobFilter::new(&None, &None);
    assert!(filter.should_include("anything.txt"));
    assert!(filter.should_include("deeply/nested/file.rs"));
    assert!(filter.should_include(""));
}

#[test]
fn glob_filter_include_single_extension() {
    let include = Some(vec!["**/*.rs".to_string()]);
    let filter = GlobFilter::new(&include, &None);
    assert!(filter.should_include("src/main.rs"));
    assert!(filter.should_include("tests/test.rs"));
    assert!(!filter.should_include("src/app.js"));
    assert!(!filter.should_include("README.md"));
}

#[test]
fn glob_filter_include_multiple_extensions() {
    let include = Some(vec!["**/*.rs".to_string(), "**/*.toml".to_string()]);
    let filter = GlobFilter::new(&include, &None);
    assert!(filter.should_include("src/main.rs"));
    assert!(filter.should_include("Cargo.toml"));
    assert!(!filter.should_include("src/app.js"));
}

#[test]
fn glob_filter_exclude_directory() {
    let exclude = Some(vec!["**/target/**".to_string()]);
    let filter = GlobFilter::new(&None, &exclude);
    assert!(filter.should_include("src/main.rs"));
    assert!(!filter.should_include("target/debug/build/foo.rs"));
    assert!(!filter.should_include("target/release/anything"));
}

#[test]
fn glob_filter_exclude_takes_priority_over_include() {
    let include = Some(vec!["**/*.rs".to_string()]);
    let exclude = Some(vec!["**/tests/**".to_string()]);
    let filter = GlobFilter::new(&include, &exclude);

    assert!(filter.should_include("src/main.rs"));
    assert!(!filter.should_include("tests/integration.rs")); // excluded by dir
    assert!(!filter.should_include("src/app.js"));            // not included
}

#[test]
fn glob_filter_exclude_specific_files() {
    let exclude = Some(vec!["**/*.lock".to_string(), "**/*.log".to_string()]);
    let filter = GlobFilter::new(&None, &exclude);
    assert!(filter.should_include("src/main.rs"));
    assert!(!filter.should_include("Cargo.lock"));
    assert!(!filter.should_include("app.log"));
}

#[test]
fn glob_filter_exclude_dotfiles() {
    let exclude = Some(vec![".*".to_string(), "**/.git/**".to_string()]);
    let filter = GlobFilter::new(&None, &exclude);
    assert!(filter.should_include("src/main.rs"));
    assert!(!filter.should_include(".gitignore"));
    assert!(!filter.should_include(".env"));
}

#[test]
fn glob_filter_empty_patterns_list() {
    let include: Option<Vec<String>> = Some(vec![]);
    let filter = GlobFilter::new(&include, &None);
    // Empty include list → built glob set matches nothing
    assert!(!filter.should_include("src/main.rs"));
}

#[test]
fn glob_filter_invalid_pattern_does_not_panic() {
    // Invalid glob pattern - should be silently skipped
    let include = Some(vec!["[invalid".to_string(), "**/*.rs".to_string()]);
    let filter = GlobFilter::new(&include, &None);
    // The valid pattern should still work
    assert!(filter.should_include("src/main.rs"));
}

#[test]
fn glob_filter_nested_path_matching() {
    let include = Some(vec!["src/**/*.rs".to_string()]);
    let filter = GlobFilter::new(&include, &None);
    assert!(filter.should_include("src/main.rs"));
    assert!(filter.should_include("src/models/mod.rs"));
    assert!(!filter.should_include("tests/test.rs"));
}

// ============================================================
// TREE GENERATOR TESTS
// ============================================================

#[test]
fn tree_generator_with_files() {
    let dir = create_test_repo();
    let tree = tree_generator::generate_tree(dir.path().to_str().unwrap(), &None, &None);

    // Should contain the root directory name
    assert!(tree.contains("/\n"), "Tree should start with root dir");

    // Should contain our files
    assert!(tree.contains("src/") || tree.contains("src"), "Tree should contain src directory");
    assert!(tree.contains("main.rs"), "Tree should contain main.rs");
    assert!(tree.contains("Cargo.toml"), "Tree should contain Cargo.toml");
}

#[test]
fn tree_generator_with_include_filter() {
    let dir = create_test_repo();
    let include = Some(vec!["**/*.rs".to_string()]);
    let tree = tree_generator::generate_tree(dir.path().to_str().unwrap(), &include, &None);

    assert!(tree.contains("main.rs"), "Filtered tree should contain .rs files");
    // JS, Python, etc. should not appear (they have no .rs extension)
    assert!(!tree.contains("app.js"), "Filtered tree should not contain .js files");
    assert!(!tree.contains("utils.py"), "Filtered tree should not contain .py files");
}

#[test]
fn tree_generator_with_exclude_filter() {
    let dir = create_test_repo();
    let exclude = Some(vec!["**/tests/**".to_string()]);
    let tree = tree_generator::generate_tree(dir.path().to_str().unwrap(), &None, &exclude);

    assert!(tree.contains("main.rs"), "Tree should still have main.rs");
    assert!(!tree.contains("test_main.rs"), "Tree should exclude test files");
}

#[test]
fn tree_generator_empty_directory() {
    let dir = TempDir::new().unwrap();
    let tree = tree_generator::generate_tree(dir.path().to_str().unwrap(), &None, &None);
    // Should have just the root name
    assert!(tree.contains("/\n"));
}

#[test]
fn tree_generator_nonexistent_path() {
    let tree = tree_generator::generate_tree("/nonexistent/path/xyz", &None, &None);
    assert!(!tree.is_empty());
}

#[test]
fn tree_generator_uses_tree_connectors() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    fs::write(dir.path().join("b.txt"), "b").unwrap();
    let tree = tree_generator::generate_tree(dir.path().to_str().unwrap(), &None, &None);

    // Should contain tree connectors
    assert!(
        tree.contains("├──") || tree.contains("└──"),
        "Tree should use visual connectors. Got: {}",
        tree
    );
}

// ============================================================
// GIT INTEGRATION TESTS
// ============================================================

#[test]
fn git_integration_non_git_dir_returns_none() {
    let dir = TempDir::new().unwrap();
    let result = git_integration::gather_git_summary(dir.path().to_str().unwrap(), 10);
    assert!(result.is_none(), "Non-git directory should return None");
}

#[test]
fn git_integration_valid_repo_returns_summary() {
    let dir = create_git_repo();
    let summary = git_integration::gather_git_summary(dir.path().to_str().unwrap(), 10);

    assert!(summary.is_some(), "Git repo should return Some(GitSummary)");
    let summary = summary.unwrap();

    assert!(!summary.recent_commits.is_empty(), "Should have recent commits");
    assert_eq!(summary.recent_commits.len(), 3, "Should have exactly 3 commits");
}

#[test]
fn git_integration_commit_structure() {
    let dir = create_git_repo();
    let summary = git_integration::gather_git_summary(dir.path().to_str().unwrap(), 10).unwrap();

    let latest = &summary.recent_commits[0];
    assert!(!latest.hash.is_empty(), "Commit hash should not be empty");
    assert!(!latest.date.is_empty(), "Commit date should not be empty");
    assert_eq!(latest.message, "Refactor main to use lib", "Latest commit message should match");
    assert!(latest.files.contains(&"src/main.rs".to_string()), "Latest commit should include src/main.rs");
}

#[test]
fn git_integration_hot_files() {
    let dir = create_git_repo();
    let summary = git_integration::gather_git_summary(dir.path().to_str().unwrap(), 100).unwrap();

    assert!(!summary.hot_files.is_empty(), "Should have hot files");

    // src/main.rs was modified in all 3 commits → should be hottest
    let hottest = &summary.hot_files[0];
    assert_eq!(hottest.0, "src/main.rs", "Hottest file should be src/main.rs");
    assert_eq!(hottest.1, 3, "src/main.rs should have 3 changes");
}

#[test]
fn git_integration_max_commits_limit() {
    let dir = create_git_repo();
    let summary = git_integration::gather_git_summary(dir.path().to_str().unwrap(), 1).unwrap();

    assert_eq!(summary.recent_commits.len(), 1, "Should respect max_commits limit");
    assert_eq!(summary.recent_commits[0].message, "Refactor main to use lib");
}

#[test]
fn git_integration_diff_stat() {
    let dir = create_git_repo();
    // Add an uncommitted change to get a non-empty diff
    fs::write(dir.path().join("src/main.rs"), "fn main() { /* changed */ }\n").unwrap();

    let diff_stat = git_integration::get_diff_stat(dir.path().to_str().unwrap());
    assert!(!diff_stat.is_empty(), "Should have working tree changes");
    assert!(diff_stat.contains("Working Tree Changes"), "Should label working tree changes");
}

#[test]
fn git_integration_full_diff() {
    let dir = create_git_repo();
    fs::write(dir.path().join("src/main.rs"), "fn main() { /* changed */ }\n").unwrap();

    let diff = git_integration::get_full_diff(dir.path().to_str().unwrap());
    assert!(!diff.is_empty(), "Should have diff content");
    assert!(diff.contains("diff --git"), "Should contain git diff header");
    assert!(diff.contains("main.rs"), "Diff should reference changed file");
}

#[test]
fn git_integration_clean_repo_empty_diff() {
    let dir = create_git_repo();
    let diff_stat = git_integration::get_diff_stat(dir.path().to_str().unwrap());
    // Clean repo with no uncommitted changes → empty diff
    assert!(diff_stat.is_empty(), "Clean repo should have empty diff stat");
}

// ============================================================
// REPO MAP TESTS
// ============================================================

#[test]
fn repo_map_extracts_rust_symbols() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);

    // Find our main.rs entry
    let main_entry = entries.iter().find(|e| e.file == "src/main.rs");
    assert!(main_entry.is_some(), "Should find src/main.rs in repo map");

    let main_entry = main_entry.unwrap();
    assert_eq!(main_entry.language, "rust");
    assert!(main_entry.tokens > 0);

    let symbol_names: Vec<&str> = main_entry.symbols.iter().map(|s| s.name.as_str()).collect();

    assert!(symbol_names.contains(&"main"), "Should find main fn");
    assert!(symbol_names.contains(&"AppConfig"), "Should find AppConfig struct");
    assert!(symbol_names.contains(&"Status"), "Should find Status enum");
    assert!(symbol_names.contains(&"Handler"), "Should find Handler trait");
    assert!(symbol_names.contains(&"AppConfig"), "Should find impl AppConfig");
    assert!(symbol_names.contains(&"helper_function"), "Should find helper_function");
    assert!(symbol_names.contains(&"async_handler"), "Should find async_handler");
}

#[test]
fn repo_map_extracts_python_symbols() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);

    let py_entry = entries.iter().find(|e| e.file == "src/utils.py");
    assert!(py_entry.is_some(), "Should find src/utils.py in repo map");

    let py_entry = py_entry.unwrap();
    assert_eq!(py_entry.language, "python");

    let symbol_names: Vec<&str> = py_entry.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(symbol_names.contains(&"process_data"), "Should find process_data");
    assert!(symbol_names.contains(&"DataProcessor"), "Should find DataProcessor class");
    assert!(symbol_names.contains(&"fetch_remote"), "Should find async def fetch_remote");
}

#[test]
fn repo_map_extracts_javascript_symbols() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);

    let js_entry = entries.iter().find(|e| e.file == "src/app.js");
    assert!(js_entry.is_some(), "Should find src/app.js in repo map");

    let js_entry = js_entry.unwrap();
    assert_eq!(js_entry.language, "javascript");

    let symbol_names: Vec<&str> = js_entry.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(symbol_names.contains(&"fetchData"), "Should find fetchData function");
    assert!(symbol_names.contains(&"ApiClient"), "Should find ApiClient class");
    assert!(symbol_names.contains(&"App"), "Should find default export App function");
}

#[test]
fn repo_map_extracts_typescript_symbols() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);

    let ts_entry = entries.iter().find(|e| e.file == "src/types.ts");
    assert!(ts_entry.is_some(), "Should find src/types.ts in repo map");

    let ts_entry = ts_entry.unwrap();
    assert_eq!(ts_entry.language, "typescript");

    let symbol_names: Vec<&str> = ts_entry.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(symbol_names.contains(&"UserDTO"), "Should find UserDTO interface");
    assert!(symbol_names.contains(&"CreateUserInput"), "Should find CreateUserInput type");
    assert!(symbol_names.contains(&"UserService"), "Should find UserService class");
    assert!(symbol_names.contains(&"validateUser"), "Should find validateUser function");
}

#[test]
fn repo_map_extracts_java_symbols() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);

    let java_entry = entries.iter().find(|e| e.file == "src/Service.java");
    assert!(java_entry.is_some(), "Should find src/Service.java in repo map");

    let java_entry = java_entry.unwrap();
    assert_eq!(java_entry.language, "java");

    let symbol_names: Vec<&str> = java_entry.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(symbol_names.contains(&"UserService"), "Should find UserService class");
    assert!(symbol_names.contains(&"UserRepository"), "Should find UserRepository interface");
    assert!(symbol_names.contains(&"Role"), "Should find Role enum");
}

#[test]
fn repo_map_extracts_go_symbols() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);

    let go_entry = entries.iter().find(|e| e.file == "src/handler.go");
    assert!(go_entry.is_some(), "Should find src/handler.go in repo map");

    let go_entry = go_entry.unwrap();
    assert_eq!(go_entry.language, "go");

    let symbol_names: Vec<&str> = go_entry.symbols.iter().map(|s| s.name.as_str()).collect();
    assert!(symbol_names.contains(&"main"), "Should find main func");
    assert!(symbol_names.contains(&"Config"), "Should find Config struct");
    assert!(symbol_names.contains(&"Handler"), "Should find Handler interface");
}

#[test]
fn repo_map_respects_include_filter() {
    let dir = create_test_repo();
    let include = Some(vec!["**/*.rs".to_string()]);
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &include, &None);

    assert!(entries.iter().all(|e| e.language == "rust"), "Should only contain Rust files");
    assert!(!entries.is_empty(), "Should have at least one entry");
}

#[test]
fn repo_map_respects_exclude_filter() {
    let dir = create_test_repo();
    let exclude = Some(vec!["**/tests/**".to_string()]);
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &exclude);

    assert!(
        !entries.iter().any(|e| e.file.starts_with("tests/")),
        "Should not contain files from tests/ directory"
    );
}

#[test]
fn repo_map_skips_non_code_files() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);

    // Markdown and TOML shouldn't appear in repo map (no language support)
    assert!(
        !entries.iter().any(|e| e.file.ends_with(".md") || e.file.ends_with(".toml")),
        "Should skip non-code files"
    );
}

#[test]
fn repo_map_sorted_by_tokens_desc() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);

    for window in entries.windows(2) {
        assert!(
            window[0].tokens >= window[1].tokens,
            "Entries should be sorted by tokens descending: {} ({}) >= {} ({})",
            window[0].file,
            window[0].tokens,
            window[1].file,
            window[1].tokens
        );
    }
}

#[test]
fn repo_map_format_output() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);
    let formatted = repo_map::format_repo_map(&entries);

    assert!(!formatted.is_empty(), "Formatted repo map should not be empty");
    // Should have file headers with language and token count
    assert!(formatted.contains("rust"), "Should mention rust language");
    assert!(formatted.contains("tokens"), "Should mention tokens");
    // Should have symbol entries
    assert!(formatted.contains("fn "), "Should list fn symbols");
}

#[test]
fn repo_map_format_empty_entries() {
    let formatted = repo_map::format_repo_map(&[]);
    assert!(formatted.is_empty(), "Empty entries should produce empty output");
}

#[test]
fn repo_map_symbol_line_numbers() {
    let dir = create_test_repo();
    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);

    for entry in &entries {
        for sym in &entry.symbols {
            assert!(sym.line > 0, "Symbol line number should be 1-indexed, got {} for {}", sym.line, sym.name);
        }
    }
}

// ============================================================
// CONTEXT FORMATTER TESTS
// ============================================================

#[test]
fn context_formatter_xml_output() {
    let dir = create_test_repo();
    let options = PackOptions {
        style: OutputStyle::Xml,
        include_tree: true,
        include_repo_map: true,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(result.content.starts_with("<repository_context>"), "XML should start with root tag");
    assert!(result.content.contains("</repository_context>"), "XML should end with root tag");
    assert!(result.content.contains("<directory_structure>"), "Should have directory tree");
    assert!(result.content.contains("<repository_map>"), "Should have repo map");
    assert!(result.content.contains("<files>"), "Should have files section");
    assert!(result.content.contains("<file path=\""), "Should have file entries");
    assert!(result.file_count > 0, "Should count files");
    assert!(result.total_tokens > 0, "Should count total tokens");
    assert_eq!(result.style, "xml");
}

#[test]
fn context_formatter_markdown_output() {
    let dir = create_test_repo();
    let options = PackOptions {
        style: OutputStyle::Markdown,
        include_tree: true,
        include_repo_map: true,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(result.content.starts_with("# Repository Context"), "MD should start with header");
    assert!(result.content.contains("## Directory Structure"), "Should have dir structure section");
    assert!(result.content.contains("## Repository Map"), "Should have repo map section");
    assert!(result.content.contains("## Files"), "Should have files section");
    assert!(result.content.contains("```"), "Should have code blocks");
    assert_eq!(result.style, "markdown");
}

#[test]
fn context_formatter_plain_output() {
    let dir = create_test_repo();
    let options = PackOptions {
        style: OutputStyle::Plain,
        include_tree: true,
        include_repo_map: true,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(result.content.contains("Repository Context"), "Plain should have header");
    assert!(result.content.contains("========"), "Plain should use separator lines");
    assert!(result.content.contains("Directory Structure"), "Should have dir structure");
    assert!(result.content.contains("File:"), "Should have file entries");
    assert_eq!(result.style, "plain");
}

#[test]
fn context_formatter_excludes_binary_files() {
    let dir = create_test_repo();
    let options = PackOptions {
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    // Binary file content should not be packed (tree/repo_map disabled to avoid false match)
    assert!(!result.content.contains("image.png"), "Should exclude PNG files from packed content");
    assert!(!result.content.contains("fake binary data"), "Should not contain binary file contents");
}

#[test]
fn context_formatter_line_numbers() {
    let dir = create_test_repo();
    let options = PackOptions {
        show_line_numbers: true,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    // Line numbers should appear as "   1 | content"
    assert!(result.content.contains(" | "), "Should have line number markers");
    assert!(result.content.contains("   1 | "), "Should start with line 1");
}

#[test]
fn context_formatter_compression() {
    let dir = create_test_repo();
    let options_full = PackOptions {
        compress: false,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };
    let options_compressed = PackOptions {
        compress: true,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let full = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options_full);
    let compressed = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options_compressed);

    // Compressed output should be smaller
    assert!(
        compressed.total_tokens <= full.total_tokens,
        "Compressed ({} tokens) should be <= full ({} tokens)",
        compressed.total_tokens,
        full.total_tokens
    );
}

#[test]
fn context_formatter_with_glob_filters() {
    let dir = create_test_repo();
    let options = PackOptions {
        include_patterns: Some(vec!["**/*.rs".to_string()]),
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    // Should only contain .rs files
    assert!(result.content.contains("main.rs"), "Should include Rust files");
    assert!(!result.content.contains("app.js"), "Should not include JS files");
    assert!(!result.content.contains("utils.py"), "Should not include Python files");
}

#[test]
fn context_formatter_no_tree_no_repo_map() {
    let dir = create_test_repo();
    let options = PackOptions {
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(!result.content.contains("<directory_structure>"), "Should not have tree");
    assert!(!result.content.contains("<repository_map>"), "Should not have repo map");
    assert!(result.content.contains("<files>"), "Should still have files");
}

#[test]
fn context_formatter_git_log_integration() {
    let dir = create_git_repo();
    let options = PackOptions {
        include_git_log: true,
        include_git_diff: false,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(result.content.contains("<git_context>"), "Should have git context section");
    assert!(result.content.contains("Recent Commits:"), "Should have recent commits");
    assert!(result.content.contains("Hot Files"), "Should have hot files section");
}

#[test]
fn context_formatter_git_diff_integration() {
    let dir = create_git_repo();
    // Make an uncommitted change
    fs::write(dir.path().join("src/main.rs"), "fn main() { /* dirty */ }\n").unwrap();

    let options = PackOptions {
        include_git_log: false,
        include_git_diff: true,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(result.content.contains("<git_context>"), "Should have git context");
    assert!(result.content.contains("Working Tree Changes"), "Should show working tree diff");
}

#[test]
fn context_formatter_output_style_from_str() {
    assert_eq!(OutputStyle::from_str("xml"), OutputStyle::Xml);
    assert_eq!(OutputStyle::from_str("XML"), OutputStyle::Xml);
    assert_eq!(OutputStyle::from_str("markdown"), OutputStyle::Markdown);
    assert_eq!(OutputStyle::from_str("md"), OutputStyle::Markdown);
    assert_eq!(OutputStyle::from_str("plain"), OutputStyle::Plain);
    assert_eq!(OutputStyle::from_str("anything_else"), OutputStyle::Plain);
    assert_eq!(OutputStyle::from_str(""), OutputStyle::Plain);
}

#[test]
fn context_formatter_empty_directory() {
    let dir = TempDir::new().unwrap();
    let options = PackOptions::default();

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert_eq!(result.file_count, 0, "Empty dir should have 0 files");
    assert!(result.total_tokens > 0, "Should still have tokens from structure/headers");
    assert_eq!(result.style, "xml");
}

#[test]
fn context_formatter_pack_options_default() {
    let defaults = PackOptions::default();
    assert_eq!(defaults.style, OutputStyle::Xml);
    assert!(!defaults.compress);
    assert!(defaults.include_patterns.is_none());
    assert!(defaults.exclude_patterns.is_none());
    assert!(!defaults.include_git_diff);
    assert!(!defaults.include_git_log);
    assert_eq!(defaults.git_log_count, 50);
    assert!(!defaults.show_line_numbers);
    assert!(!defaults.remove_empty_lines);
    assert!(defaults.include_tree);
    assert!(defaults.include_repo_map);
}

#[test]
fn context_formatter_file_count_accuracy() {
    let dir = create_test_repo();
    let options = PackOptions {
        include_patterns: Some(vec!["**/*.rs".to_string()]),
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    // We created: src/main.rs, src/models/mod.rs, tests/test_main.rs = 3 .rs files
    assert_eq!(result.file_count, 3, "Should find exactly 3 .rs files");
}

#[test]
fn context_formatter_remove_empty_lines() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("test.rs"),
        "line1\n\n\nline4\n\nline6\n",
    )
    .unwrap();

    let options = PackOptions {
        remove_empty_lines: true,
        compress: false,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    // The file content in the output should NOT have consecutive empty lines
    // (but the surrounding XML structure will have its own formatting)
    assert!(result.file_count == 1);
    // Check that the content doesn't have double newlines from the file
    let file_content_region = result.content.split("tokens=\"").nth(1).unwrap_or("");
    // Within the file data, empty lines should be removed
    assert!(!file_content_region.contains("\n\n\n"), "Should not have triple newlines from file content");
}

// ============================================================
// COMPRESSION LOGIC TESTS (more granular)
// ============================================================

#[test]
fn compression_keeps_imports_rust() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("main.rs"),
        r#"use std::collections::HashMap;
pub use crate::models;

pub fn process(data: &str) -> String {
    let mut result = String::new();
    result.push_str(data);
    result
}
"#,
    )
    .unwrap();

    let options = PackOptions {
        compress: true,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(result.content.contains("use std::collections::HashMap"), "Should keep imports");
    assert!(result.content.contains("pub use crate::models"), "Should keep pub use");
    assert!(result.content.contains("pub fn process"), "Should keep function signature");
}

#[test]
fn compression_keeps_imports_python() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("app.py"),
        r#"import os
from pathlib import Path

def process_data(items):
    result = []
    for item in items:
        result.append(item)
    return result
"#,
    )
    .unwrap();

    let options = PackOptions {
        compress: true,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(result.content.contains("import os"), "Should keep import");
    assert!(result.content.contains("from pathlib import Path"), "Should keep from-import");
    assert!(result.content.contains("def process_data"), "Should keep function signature");
}

#[test]
fn compression_keeps_imports_javascript() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("app.js"),
        r#"import React from 'react';
import { useState } from 'react';

export function Component() {
    const [state, setState] = useState(0);
    return <div>{state}</div>;
}
"#,
    )
    .unwrap();

    let options = PackOptions {
        compress: true,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(result.content.contains("import React from 'react'"), "Should keep import");
    assert!(result.content.contains("import { useState }"), "Should keep named import");
}

// ============================================================
// EDGE CASES AND ROBUSTNESS
// ============================================================

#[test]
fn handles_files_with_no_extension() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("Makefile"), "all:\n\techo hello\n").unwrap();
    fs::write(dir.path().join("Dockerfile"), "FROM ubuntu\n").unwrap();

    let options = PackOptions {
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    // These files should still be included (not binary, just no recognized code language)
    assert!(result.file_count >= 1, "Should include extensionless files");
}

#[test]
fn handles_deeply_nested_files() {
    let dir = TempDir::new().unwrap();
    let deep_path = dir.path().join("a/b/c/d/e/f");
    fs::create_dir_all(&deep_path).unwrap();
    fs::write(deep_path.join("deep.rs"), "fn deep() {}\n").unwrap();

    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);
    assert!(entries.iter().any(|e| e.file.contains("deep.rs")), "Should find deeply nested files");
}

#[test]
fn handles_empty_files() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("empty.rs"), "").unwrap();

    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);
    let empty_entry = entries.iter().find(|e| e.file == "empty.rs");
    assert!(empty_entry.is_some(), "Should include empty files");
    assert_eq!(empty_entry.unwrap().tokens, 0, "Empty file should have 0 tokens");
    assert!(empty_entry.unwrap().symbols.is_empty(), "Empty file should have no symbols");
}

#[test]
fn handles_large_file() {
    let dir = TempDir::new().unwrap();
    // Generate a large Rust file
    let mut content = String::new();
    for i in 0..100 {
        content.push_str(&format!(
            "pub fn function_{}(x: i32) -> i32 {{\n    x + {}\n}}\n\n",
            i, i
        ));
    }
    fs::write(dir.path().join("large.rs"), &content).unwrap();

    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);
    let large_entry = entries.iter().find(|e| e.file == "large.rs").unwrap();

    assert_eq!(large_entry.symbols.len(), 100, "Should find all 100 functions");
    assert!(large_entry.tokens > 100, "Should have substantial token count");
}

#[test]
fn handles_special_characters_in_filenames() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("file-with-dashes.rs"), "fn hello() {}\n").unwrap();
    fs::write(dir.path().join("file_with_underscores.rs"), "fn world() {}\n").unwrap();

    let entries = repo_map::generate_repo_map(dir.path().to_str().unwrap(), &None, &None);
    assert!(entries.iter().any(|e| e.file.contains("dashes")));
    assert!(entries.iter().any(|e| e.file.contains("underscores")));
}

#[test]
fn tree_generator_preserves_directory_hierarchy() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("src/models")).unwrap();
    fs::create_dir_all(dir.path().join("src/handlers")).unwrap();
    fs::write(dir.path().join("src/models/user.rs"), "struct User;").unwrap();
    fs::write(dir.path().join("src/handlers/api.rs"), "fn api() {}").unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();

    let tree = tree_generator::generate_tree(dir.path().to_str().unwrap(), &None, &None);

    // Verify hierarchy is maintained
    assert!(tree.contains("src/"), "Should show src directory");
    assert!(tree.contains("models/"), "Should show models subdirectory");
    assert!(tree.contains("handlers/"), "Should show handlers subdirectory");
    assert!(tree.contains("user.rs"), "Should show user.rs file");
    assert!(tree.contains("api.rs"), "Should show api.rs file");
    assert!(tree.contains("main.rs"), "Should show main.rs file");
}

// ============================================================
// MODELS TESTS
// ============================================================

#[test]
fn pack_output_serialization() {
    use ai_platform::models::PackOutput;

    let output = PackOutput {
        content: "test content".to_string(),
        total_tokens: 100,
        file_count: 5,
        style: "xml".to_string(),
    };

    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("test content"));
    assert!(json.contains("100"));
    assert!(json.contains("xml"));

    let deserialized: PackOutput = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.total_tokens, 100);
    assert_eq!(deserialized.file_count, 5);
}

#[test]
fn pack_request_serialization() {
    use ai_platform::models::PackRequest;

    let request = PackRequest {
        repo_path: "/tmp/test".to_string(),
        style: Some("markdown".to_string()),
        compress: Some(true),
        include_patterns: Some(vec!["**/*.rs".to_string()]),
        exclude_patterns: None,
        include_git_diff: Some(false),
        include_git_log: Some(true),
        git_log_count: Some(20),
        show_line_numbers: Some(false),
        include_tree: Some(true),
        include_repo_map: Some(true),
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: PackRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.repo_path, "/tmp/test");
    assert_eq!(deserialized.style, Some("markdown".to_string()));
    assert_eq!(deserialized.compress, Some(true));
    assert_eq!(deserialized.git_log_count, Some(20));
}

#[test]
fn pack_request_minimal_fields() {
    use ai_platform::models::PackRequest;

    let json = r#"{"repo_path": "/tmp/minimal"}"#;
    let request: PackRequest = serde_json::from_str(json).unwrap();
    assert_eq!(request.repo_path, "/tmp/minimal");
    assert!(request.style.is_none());
    assert!(request.compress.is_none());
    assert!(request.include_patterns.is_none());
}

#[test]
fn git_summary_serialization() {
    use ai_platform::models::{GitSummary, CommitInfo};

    let summary = GitSummary {
        recent_commits: vec![CommitInfo {
            hash: "abc12345".to_string(),
            date: "2024-01-01".to_string(),
            message: "Initial commit".to_string(),
            files: vec!["src/main.rs".to_string()],
        }],
        diff_stat: "1 file changed".to_string(),
        hot_files: vec![("src/main.rs".to_string(), 5)],
    };

    let json = serde_json::to_string(&summary).unwrap();
    let deserialized: GitSummary = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.recent_commits.len(), 1);
    assert_eq!(deserialized.recent_commits[0].hash, "abc12345");
    assert_eq!(deserialized.hot_files[0].1, 5);
}

// ============================================================
// MULTI-FORMAT CONSISTENCY TESTS
// ============================================================

#[test]
fn all_formats_have_same_file_count() {
    let dir = create_test_repo();

    let xml = context_formatter::pack_repository(
        dir.path().to_str().unwrap(),
        &PackOptions { style: OutputStyle::Xml, include_tree: false, include_repo_map: false, ..PackOptions::default() },
    );
    let md = context_formatter::pack_repository(
        dir.path().to_str().unwrap(),
        &PackOptions { style: OutputStyle::Markdown, include_tree: false, include_repo_map: false, ..PackOptions::default() },
    );
    let plain = context_formatter::pack_repository(
        dir.path().to_str().unwrap(),
        &PackOptions { style: OutputStyle::Plain, include_tree: false, include_repo_map: false, ..PackOptions::default() },
    );

    assert_eq!(xml.file_count, md.file_count, "XML and Markdown should have same file count");
    assert_eq!(md.file_count, plain.file_count, "Markdown and Plain should have same file count");
    assert!(xml.file_count > 0, "Should have files");
}

#[test]
fn compressed_output_retains_definitions() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("lib.rs"),
        r#"use std::io;

pub struct Config {
    pub name: String,
    pub value: i32,
}

pub fn create_config(name: &str) -> Config {
    Config {
        name: name.to_string(),
        value: 42,
    }
}

pub enum State {
    Active,
    Inactive,
}

pub trait Processor {
    fn process(&self);
}

impl Config {
    pub fn validate(&self) -> bool {
        self.value > 0
    }
}
"#,
    )
    .unwrap();

    let options = PackOptions {
        compress: true,
        include_tree: false,
        include_repo_map: false,
        ..PackOptions::default()
    };

    let result = context_formatter::pack_repository(dir.path().to_str().unwrap(), &options);

    assert!(result.content.contains("use std::io"), "Compression should keep imports");
    assert!(result.content.contains("pub struct Config"), "Compression should keep struct defs");
    assert!(result.content.contains("pub fn create_config"), "Compression should keep fn signatures");
    assert!(result.content.contains("pub enum State"), "Compression should keep enum defs");
    assert!(result.content.contains("pub trait Processor"), "Compression should keep trait defs");
    assert!(result.content.contains("impl Config"), "Compression should keep impl blocks");
}
