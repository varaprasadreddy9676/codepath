use tracing::{info, warn};
use std::process::Command;
use serde_json::Value;

pub async fn parse_repository(repo_url: &str) {
    info!("Initializing JavaParser AST extraction for repo: {}", repo_url);
    // STUB: Queue extraction logic over .java file trees
}

pub fn extract_java_ast(file_path: &str) -> Option<Value> {
    // Trigger the specialized Maven-compiled JavaParser worker dynamically out-of-band
    // passing the active request payload directly as arguments
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
        // Without the heavy JVM dependencies pre-built, the sub-process router should securely 
        // collapse back to None instead of panicking the primary orchestrator
        let extraction = extract_java_ast("NonExistentConfig.java");
        assert!(extraction.is_none()); 
    }
}
