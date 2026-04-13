use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;

use crate::model::{ContextFile, ContextSet, DirectoryContext, Layer};

/// Walk the project tree and collect all directories that contain
/// `*.context.md` files, returning a structured `ContextSet`.
pub fn discover(root: &Path) -> Result<ContextSet> {
    let mut raw: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

    walk(root, root, &mut raw)?;

    let directories = raw
        .into_iter()
        .map(|(relative_dir, file_entries)| {
            let dir = if relative_dir.is_empty() {
                root.to_path_buf()
            } else {
                root.join(&relative_dir)
            };

            let mut files: Vec<ContextFile> = file_entries
                .into_iter()
                .map(|(filename, content)| {
                    let relative_path = if relative_dir.is_empty() {
                        filename.clone()
                    } else {
                        format!("{}/{}", relative_dir, filename)
                    };
                    let layer = Layer::from_filename(&filename);
                    ContextFile {
                        relative_path,
                        filename,
                        layer,
                        content,
                    }
                })
                .collect();

            files.sort_by_key(|f| f.layer.sort_priority());

            DirectoryContext {
                dir,
                relative_dir,
                files,
            }
        })
        .collect();

    Ok(ContextSet {
        root: root.to_path_buf(),
        directories,
    })
}

fn walk(
    dir: &Path,
    root: &Path,
    results: &mut BTreeMap<String, Vec<(String, String)>>,
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

        if path.is_dir() {
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            walk(&path, root, results)?;
            continue;
        }

        if is_context_file(&name) {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("failed to read {}: {}", path.display(), e))?;

            let relative_dir = dir
                .strip_prefix(root)
                .unwrap_or(dir)
                .to_string_lossy()
                .to_string();

            results
                .entry(relative_dir)
                .or_default()
                .push((name, content));
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

        let ctx_set = discover(tmp.path()).unwrap();
        assert_eq!(ctx_set.directories.len(), 1);
        assert_eq!(ctx_set.directories[0].files.len(), 2);

        let filenames: Vec<&str> = ctx_set.directories[0]
            .files
            .iter()
            .map(|f| f.filename.as_str())
            .collect();
        assert!(filenames.contains(&"project.context.md"));
        assert!(filenames.contains(&"corporate.context.md"));
    }

    #[test]
    fn discover_finds_nested_context_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("project.context.md"), "# Root").unwrap();
        let domain_dir = tmp.path().join("src").join("billing");
        fs::create_dir_all(&domain_dir).unwrap();
        fs::write(domain_dir.join("domain.context.md"), "# Billing").unwrap();

        let ctx_set = discover(tmp.path()).unwrap();
        assert_eq!(ctx_set.directories.len(), 2);
    }

    #[test]
    fn discover_skips_hidden_dirs() {
        let tmp = TempDir::new().unwrap();
        let hidden = tmp.path().join(".hidden");
        fs::create_dir_all(&hidden).unwrap();
        fs::write(hidden.join("domain.context.md"), "# Hidden").unwrap();

        let ctx_set = discover(tmp.path()).unwrap();
        assert!(ctx_set.directories.is_empty());
    }

    #[test]
    fn discover_skips_node_modules() {
        let tmp = TempDir::new().unwrap();
        let nm = tmp.path().join("node_modules").join("pkg");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("domain.context.md"), "# Pkg").unwrap();

        let ctx_set = discover(tmp.path()).unwrap();
        assert!(ctx_set.directories.is_empty());
    }

    #[test]
    fn discover_skips_target_dir() {
        let tmp = TempDir::new().unwrap();
        let target = tmp.path().join("target").join("debug");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("implementation.context.md"), "# Build").unwrap();

        let ctx_set = discover(tmp.path()).unwrap();
        assert!(ctx_set.directories.is_empty());
    }

    #[test]
    fn discover_ignores_non_context_md_files() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("README.md"), "# Readme").unwrap();
        fs::write(tmp.path().join("CLAUDE.md"), "# Claude").unwrap();
        fs::write(tmp.path().join("notes.md"), "# Notes").unwrap();

        let ctx_set = discover(tmp.path()).unwrap();
        assert!(ctx_set.directories.is_empty());
    }

    #[test]
    fn discover_empty_directory() {
        let tmp = TempDir::new().unwrap();
        let ctx_set = discover(tmp.path()).unwrap();
        assert!(ctx_set.directories.is_empty());
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

    #[test]
    fn files_are_sorted_by_layer_priority() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("project.context.md"), "# Project").unwrap();
        fs::write(tmp.path().join("nrs.context.md"), "# NRS").unwrap();
        fs::write(tmp.path().join("corporate.context.md"), "# Corp").unwrap();

        let ctx_set = discover(tmp.path()).unwrap();
        let filenames: Vec<&str> = ctx_set.directories[0]
            .files
            .iter()
            .map(|f| f.filename.as_str())
            .collect();
        assert_eq!(
            filenames,
            vec!["nrs.context.md", "corporate.context.md", "project.context.md"]
        );
    }

    #[test]
    fn relative_paths_are_correct() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("project.context.md"), "# Root").unwrap();
        let billing = tmp.path().join("src").join("billing");
        fs::create_dir_all(&billing).unwrap();
        fs::write(billing.join("domain.context.md"), "# Billing").unwrap();

        let ctx_set = discover(tmp.path()).unwrap();

        let root_file = &ctx_set.directories[0].files[0];
        assert_eq!(root_file.relative_path, "project.context.md");

        let nested_file = &ctx_set.directories[1].files[0];
        assert_eq!(nested_file.relative_path, "src/billing/domain.context.md");
    }

    #[test]
    fn all_files_iterates_across_directories() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("project.context.md"), "# Root").unwrap();
        let billing = tmp.path().join("src").join("billing");
        fs::create_dir_all(&billing).unwrap();
        fs::write(billing.join("domain.context.md"), "# Billing").unwrap();

        let ctx_set = discover(tmp.path()).unwrap();
        let all: Vec<&str> = ctx_set
            .all_files()
            .map(|f| f.filename.as_str())
            .collect();
        assert_eq!(all.len(), 2);
    }
}
