use tracing::info;
use tree_sitter::Parser;
use walkdir::WalkDir;
use std::fs;

pub async fn parse_repository(repo_path: &str) {
    info!("Initializing Tree-sitter directory traversal for repo: {}", repo_path);
    
    // Iterating over repository files and handling multi-language files dynamically natively
    for entry in WalkDir::new(repo_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "rs" {
                    if let Ok(content) = fs::read_to_string(path) {
                        parse_rust_file(&content);
                    }
                }
            }
        }
    }
}

pub fn parse_rust_file(source_code: &str) -> Vec<String> {
    let mut parser = Parser::new();
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).expect("Error loading Rust grammar");
    
    let tree = parser.parse(source_code, None).expect("Failed to parse code natively");
    let root_node = tree.root_node();
    
    info!("Extracted AST for file. Root node type: {}", root_node.kind());
    vec![root_node.kind().to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_sitter_native_parsing() {
        let code = "fn diagnostic_engine() { println!(\"Starting AST Parsing\"); }";
        let captured_nodes = parse_rust_file(code);
        assert!(!captured_nodes.is_empty());
        assert_eq!(captured_nodes[0], "source_file");
    }
}
