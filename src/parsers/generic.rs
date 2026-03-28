use tracing::info;
use tree_sitter::Parser;

pub async fn parse_repository(repo_url: &str) {
    info!("Initializing Tree-sitter traversal for repo: {}", repo_url);
    // STUB: Real implementation will walk directory paths and map extensions to grammars
}

pub fn parse_rust_file(source_code: &str) -> Vec<String> {
    let mut parser = Parser::new();
    
    // Modern tree-sitter-rust language binding
    parser.set_language(&tree_sitter_rust::LANGUAGE.into()).expect("Error loading Rust grammar");
    
    let tree = parser.parse(source_code, None).expect("Failed to parse code natively");
    let root_node = tree.root_node();
    
    info!("Extracted AST for file. Root node type: {}", root_node.kind());
    
    // For this MVP, we are isolating the root node type to prove integration connection
    // Future iterations will recursively chunk methods and structs into the Qdrant store
    vec![root_node.kind().to_string()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_sitter_native_parsing() {
        let code = "fn diagnostic_engine() { println!(\"Starting AST Parsing\"); }";
        let captured_nodes = parse_rust_file(code);
        
        // Ensure tree-sitter correctly chunked the text into a source file
        assert!(!captured_nodes.is_empty());
        assert_eq!(captured_nodes[0], "source_file");
    }
}
