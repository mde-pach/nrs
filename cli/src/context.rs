use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Context files found in a single directory.
pub struct DirectoryContext {
    pub dir: PathBuf,
    /// Context file contents, keyed by filename (e.g., "corporate.context.md", "CONTEXT.md").
    /// Ordered alphabetically so output is deterministic.
    pub files: BTreeMap<String, String>,
}

/// Walk the project tree and collect all directories that contain *.context.md or CONTEXT.md files.
pub fn discover(root: &Path) -> Result<Vec<DirectoryContext>> {
    let mut results: BTreeMap<PathBuf, BTreeMap<String, String>> = BTreeMap::new();

    walk(root, &mut results, root)?;

    Ok(results
        .into_iter()
        .map(|(dir, files)| DirectoryContext { dir, files })
        .collect())
}

fn walk(
    dir: &Path,
    results: &mut BTreeMap<PathBuf, BTreeMap<String, String>>,
    root: &Path,
) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => return Ok(()),
        Err(e) => return Err(anyhow::anyhow!("failed to read {}: {}", dir.display(), e)),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden directories and node_modules
        if path.is_dir() {
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            walk(&path, results, root)?;
            continue;
        }

        if is_context_file(&name) {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("failed to read {}: {}", path.display(), e))?;
            results
                .entry(dir.to_path_buf())
                .or_default()
                .insert(name, content);
        }
    }

    Ok(())
}

fn is_context_file(name: &str) -> bool {
    name.ends_with(".context.md")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discover_finds_root_context_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("project.context.md"), "# Project").unwrap();
        fs::write(tmp.path().join("corporate.context.md"), "# Corp").unwrap();

        let results = discover(tmp.path()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].files.len(), 2);
        assert!(results[0].files.contains_key("project.context.md"));
        assert!(results[0].files.contains_key("corporate.context.md"));
    }

    #[test]
    fn discover_finds_nested_context_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("project.context.md"), "# Root").unwrap();
        let domain_dir = tmp.path().join("src").join("billing");
        fs::create_dir_all(&domain_dir).unwrap();
        fs::write(domain_dir.join("domain.context.md"), "# Billing").unwrap();

        let results = discover(tmp.path()).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn discover_skips_hidden_dirs() {
        let tmp = TempDir::new().unwrap();
        let hidden = tmp.path().join(".hidden");
        fs::create_dir_all(&hidden).unwrap();
        fs::write(hidden.join("domain.context.md"), "# Hidden").unwrap();

        let results = discover(tmp.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn discover_skips_node_modules() {
        let tmp = TempDir::new().unwrap();
        let nm = tmp.path().join("node_modules").join("pkg");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("domain.context.md"), "# Pkg").unwrap();

        let results = discover(tmp.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn discover_skips_target_dir() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target").join("debug");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("implementation.context.md"), "# Build").unwrap();

        let results = discover(tmp.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn discover_ignores_non_context_md_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("README.md"), "# Readme").unwrap();
        fs::write(tmp.path().join("CLAUDE.md"), "# Claude").unwrap();
        fs::write(tmp.path().join("notes.md"), "# Notes").unwrap();

        let results = discover(tmp.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn discover_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let results = discover(tmp.path()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn is_context_file_matches_correctly() {
        assert!(is_context_file("project.context.md"));
        assert!(is_context_file("domain.context.md"));
        assert!(is_context_file("nrs.context.md"));
        assert!(is_context_file("custom.context.md"));
        assert!(!is_context_file("README.md"));
        assert!(!is_context_file("CLAUDE.md"));
        assert!(!is_context_file("context.md"));
        assert!(!is_context_file("something.md"));
    }
}
