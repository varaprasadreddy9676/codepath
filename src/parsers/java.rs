use tracing::{info, warn};
use std::process::Command;
use serde_json::Value;
use walkdir::WalkDir;

pub async fn parse_repository(repo_path: &str) {
    info!("Initializing JavaParser AST extraction traversing: {}", repo_path);
    
    for entry in WalkDir::new(repo_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("java") {
            let path_str = path.to_string_lossy();
            let _ast = extract_java_ast(&path_str);
        }
    }
}

pub fn extract_java_ast(file_path: &str) -> Option<Value> {
    let output = Command::new("java")
        .arg("-cp")
        .arg("workers/java-parser/target/classes:workers/java-parser/lib/*")
        .arg("com.codepath.JavaASTExtractor")
        .arg(file_path)
        .output();

    match output {
        Ok(cmd_output) => {
            if cmd_output.status.success() {
                let stdout_text = String::from_utf8_lossy(&cmd_output.stdout);
                info!("JavaParser worker completed effectively.");
                serde_json::from_str(&stdout_text).ok()
            } else {
                let stderr_text = String::from_utf8_lossy(&cmd_output.stderr);
                warn!("JavaParser worker process tripped an internal error: {}", stderr_text);
                None
            }
        },
        Err(err) => {
            warn!("Failed to initialize Java subsystem host. Is Java installed locally? Error: {}", err);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_parser_graceful_routing() {
        let extraction = extract_java_ast("NonExistentConfig.java");
        assert!(extraction.is_none()); 
    }
}
