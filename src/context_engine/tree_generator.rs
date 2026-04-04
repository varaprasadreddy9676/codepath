/// Directory tree generator that produces a visual file tree
/// similar to the `tree` command, respecting gitignore rules.

use ignore::WalkBuilder;
use std::collections::BTreeMap;
use std::path::Path;

use super::glob_filter::GlobFilter;

/// A node in the directory tree
enum TreeNode {
    File(String),
    Dir(String, BTreeMap<String, TreeNode>),
}

/// Generate a visual directory tree string for a repository path.
/// Respects .gitignore and optionally applies include/exclude glob patterns.
pub fn generate_tree(
    root: &str,
    include_patterns: &Option<Vec<String>>,
    exclude_patterns: &Option<Vec<String>>,
) -> String {
    let root_path = Path::new(root);
    let filter = GlobFilter::new(include_patterns, exclude_patterns);

    let mut tree: BTreeMap<String, TreeNode> = BTreeMap::new();

    let walker = WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path == root_path {
            continue;
        }

        let rel = match path.strip_prefix(root_path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let rel_str = rel.to_string_lossy().to_string();

        if !filter.should_include(&rel_str) {
            continue;
        }

        let parts: Vec<&str> = rel_str.split('/').collect();
        insert_into_tree(&mut tree, &parts, path.is_dir());
    }

    let root_name = root_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.to_string());

    let mut output = format!("{}/\n", root_name);
    render_tree(&tree, &mut output, "");
    output
}

fn insert_into_tree(tree: &mut BTreeMap<String, TreeNode>, parts: &[&str], is_dir: bool) {
    if parts.is_empty() {
        return;
    }

    let name = parts[0].to_string();

    if parts.len() == 1 {
        if is_dir {
            tree.entry(name.clone())
                .or_insert_with(|| TreeNode::Dir(name, BTreeMap::new()));
        } else {
            tree.entry(name.clone())
                .or_insert(TreeNode::File(name));
        }
    } else {
        let entry = tree
            .entry(name.clone())
            .or_insert_with(|| TreeNode::Dir(name, BTreeMap::new()));
        if let TreeNode::Dir(_, ref mut children) = entry {
            insert_into_tree(children, &parts[1..], is_dir);
        }
    }
}

fn render_tree(tree: &BTreeMap<String, TreeNode>, output: &mut String, prefix: &str) {
    let entries: Vec<_> = tree.iter().collect();
    let total = entries.len();

    for (i, (_, node)) in entries.iter().enumerate() {
        let is_last = i == total - 1;
        let connector = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };

        match node {
            TreeNode::File(name) => {
                output.push_str(&format!("{}{}{}\n", prefix, connector, name));
            }
            TreeNode::Dir(name, children) => {
                output.push_str(&format!("{}{}{}/\n", prefix, connector, name));
                render_tree(children, output, &format!("{}{}", prefix, child_prefix));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_generate_tree_nonexistent() {
        let tree = generate_tree("/nonexistent/path/that/does/not/exist", &None, &None);
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_insert_into_tree_single_file() {
        let mut tree = BTreeMap::new();
        insert_into_tree(&mut tree, &["file.rs"], false);
        assert!(tree.contains_key("file.rs"));
    }

    #[test]
    fn test_insert_into_tree_nested() {
        let mut tree = BTreeMap::new();
        insert_into_tree(&mut tree, &["src", "main.rs"], false);
        assert!(tree.contains_key("src"));
        if let Some(TreeNode::Dir(_, children)) = tree.get("src") {
            assert!(children.contains_key("main.rs"));
        } else {
            panic!("src should be a directory");
        }
    }

    #[test]
    fn test_insert_empty_parts() {
        let mut tree = BTreeMap::new();
        insert_into_tree(&mut tree, &[], false);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_render_tree_format() {
        let mut tree = BTreeMap::new();
        insert_into_tree(&mut tree, &["alpha.rs"], false);
        insert_into_tree(&mut tree, &["beta.rs"], false);

        let mut output = String::new();
        render_tree(&tree, &mut output, "");

        assert!(output.contains("├── alpha.rs\n"));
        assert!(output.contains("└── beta.rs\n"));
    }

    #[test]
    fn test_generate_tree_real_dir() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();
        fs::write(dir.path().join("README.md"), "# Hi").unwrap();

        let tree = generate_tree(dir.path().to_str().unwrap(), &None, &None);
        assert!(tree.contains("src/"));
        assert!(tree.contains("main.rs"));
        assert!(tree.contains("README.md"));
    }
}
