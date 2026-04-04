/// Glob-based file filtering for include/exclude patterns.
/// Supports standard glob syntax: *, **, ?, [chars].

use globset::{Glob, GlobSet, GlobSetBuilder};

pub struct GlobFilter {
    include: Option<GlobSet>,
    exclude: Option<GlobSet>,
}

impl GlobFilter {
    pub fn new(
        include_patterns: &Option<Vec<String>>,
        exclude_patterns: &Option<Vec<String>>,
    ) -> Self {
        let include = include_patterns.as_ref().map(|patterns| {
            let mut builder = GlobSetBuilder::new();
            for pat in patterns {
                if let Ok(glob) = Glob::new(pat) {
                    builder.add(glob);
                }
            }
            builder.build().unwrap_or_else(|_| GlobSetBuilder::new().build().unwrap())
        });

        let exclude = exclude_patterns.as_ref().map(|patterns| {
            let mut builder = GlobSetBuilder::new();
            for pat in patterns {
                if let Ok(glob) = Glob::new(pat) {
                    builder.add(glob);
                }
            }
            builder.build().unwrap_or_else(|_| GlobSetBuilder::new().build().unwrap())
        });

        Self { include, exclude }
    }

    /// Check if a relative file path should be included based on the filter rules.
    pub fn should_include(&self, rel_path: &str) -> bool {
        // If exclude patterns exist and match, exclude the file
        if let Some(ref exclude) = self.exclude {
            if exclude.is_match(rel_path) {
                return false;
            }
        }

        // If include patterns exist, file must match at least one
        if let Some(ref include) = self.include {
            return include.is_match(rel_path);
        }

        // No include patterns means include everything (that wasn't excluded)
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_filters() {
        let filter = GlobFilter::new(&None, &None);
        assert!(filter.should_include("src/main.rs"));
        assert!(filter.should_include("anything.txt"));
    }

    #[test]
    fn test_include_only() {
        let include = Some(vec!["**/*.rs".to_string()]);
        let filter = GlobFilter::new(&include, &None);
        assert!(filter.should_include("src/main.rs"));
        assert!(!filter.should_include("src/main.js"));
    }

    #[test]
    fn test_exclude_only() {
        let exclude = Some(vec!["**/target/**".to_string()]);
        let filter = GlobFilter::new(&None, &exclude);
        assert!(filter.should_include("src/main.rs"));
        assert!(!filter.should_include("target/debug/build"));
    }

    #[test]
    fn test_include_and_exclude() {
        let include = Some(vec!["**/*.rs".to_string()]);
        let exclude = Some(vec!["**/test*".to_string()]);
        let filter = GlobFilter::new(&include, &exclude);
        assert!(filter.should_include("src/main.rs"));
        assert!(!filter.should_include("src/test_main.rs"));
        assert!(!filter.should_include("src/main.js"));
    }
}
