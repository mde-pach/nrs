use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::markdown;
use crate::model::ContextSet;
use crate::validate::{Finding, Validator, ValidatorScope};

pub struct OrphanDocsValidator;

impl Validator for OrphanDocsValidator {
    fn name(&self) -> &str {
        "orphan_docs"
    }

    fn scope(&self) -> ValidatorScope {
        ValidatorScope::Global
    }

    fn check_all(&self, ctx_set: &ContextSet) -> anyhow::Result<Vec<Finding>> {
        Ok(check(ctx_set))
    }
}

/// Check that every markdown file under `docs/` is referenced from at least
/// one `*.context.md` file.  Unreferenced docs are invisible to agents — they
/// can only discover documentation through context file links.
pub fn check(ctx_set: &ContextSet) -> Vec<Finding> {
    let docs_dir = ctx_set.root.join("docs");
    if !docs_dir.is_dir() {
        return Vec::new();
    }

    // 1. Collect all .md files under docs/, as relative paths from root.
    let doc_files = collect_docs(&docs_dir, &ctx_set.root);
    if doc_files.is_empty() {
        return Vec::new();
    }

    // 2. Collect every markdown link target from every context file.
    let mut referenced: HashSet<String> = HashSet::new();
    for dir_ctx in &ctx_set.directories {
        for file in &dir_ctx.files {
            for line in file.content.lines() {
                for target in markdown::extract_md_links(line) {
                    // Skip external URLs and anchors
                    if target.starts_with("http://")
                        || target.starts_with("https://")
                        || target.starts_with('#')
                    {
                        continue;
                    }

                    // Strip anchor fragments (e.g. "docs/auth.md#tokens" → "docs/auth.md")
                    let clean = target.split('#').next().unwrap_or(target);

                    // Resolve from context dir
                    let from_ctx = normalise_joined(&dir_ctx.dir, clean, &ctx_set.root);
                    if let Some(rel) = from_ctx {
                        referenced.insert(rel);
                    }
                    // Resolve from project root
                    let from_root = normalise_joined(&ctx_set.root, clean, &ctx_set.root);
                    if let Some(rel) = from_root {
                        referenced.insert(rel);
                    }
                }
            }
        }
    }

    // 3. Report docs that are not referenced.
    let mut findings = Vec::new();
    for doc in &doc_files {
        if !referenced.contains(doc) {
            findings.push(Finding::warning(
                doc.clone(),
                "doc is not referenced from any context file — agents cannot discover it",
            ));
        }
    }

    findings
}

/// Join `base` and `relative`, collapse `.` / `..` components, and return a
/// normalised path relative to `root`.  Returns `None` if the result escapes
/// `root` or doesn't start with `docs/`.
fn normalise_joined(base: &Path, relative: &str, root: &Path) -> Option<String> {
    let joined = base.join(relative);
    let collapsed = collapse(&joined);

    let rel = collapsed.strip_prefix(root).ok()?;
    let s = rel.to_string_lossy().replace('\\', "/");

    // Only care about docs/ paths
    if s.starts_with("docs/") {
        Some(s)
    } else {
        None
    }
}

/// Collapse `.` and `..` components without touching the filesystem.
fn collapse(path: &Path) -> PathBuf {
    let mut parts: Vec<&std::ffi::OsStr> = Vec::new();
    for c in path.components() {
        match c {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                parts.pop();
            }
            _ => parts.push(c.as_os_str()),
        }
    }
    parts.iter().collect()
}

/// Recursively collect all .md files under a directory, returning paths
/// relative to `root`.
fn collect_docs(dir: &Path, root: &Path) -> Vec<String> {
    let mut results = Vec::new();
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return results,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            if !name.starts_with('.') {
                results.extend(collect_docs(&path, root));
            }
            continue;
        }

        if name.ends_with(".md") {
            if let Ok(rel) = path.strip_prefix(root) {
                results.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    results.sort();
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ContextFile, DirectoryContext, Layer};
    use tempfile::TempDir;

    fn make_ctx_set(root: &Path, dirs: Vec<(&str, Vec<(&str, &str)>)>) -> ContextSet {
        let directories = dirs
            .into_iter()
            .map(|(rel_dir, file_entries)| {
                let d = if rel_dir.is_empty() {
                    root.to_path_buf()
                } else {
                    root.join(rel_dir)
                };
                std::fs::create_dir_all(&d).unwrap();

                let files = file_entries
                    .into_iter()
                    .map(|(name, content)| {
                        std::fs::write(d.join(name), content).unwrap();
                        let relative_path = if rel_dir.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}/{}", rel_dir, name)
                        };
                        ContextFile {
                            relative_path,
                            filename: name.to_string(),
                            layer: Layer::from_filename(name),
                            content: content.to_string(),
                        }
                    })
                    .collect();

                DirectoryContext {
                    dir: d,
                    relative_dir: rel_dir.to_string(),
                    files,
                }
            })
            .collect();

        ContextSet {
            root: root.to_path_buf(),
            directories,
        }
    }

    #[test]
    fn referenced_doc_is_clean() {
        let tmp = TempDir::new().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("testing.md"), "# Testing").unwrap();

        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![(
                "",
                vec![("project.context.md", "- [Testing](docs/testing.md)")],
            )],
        );

        let findings = check(&ctx_set);
        assert!(findings.is_empty());
    }

    #[test]
    fn orphan_doc_is_warning() {
        let tmp = TempDir::new().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("testing.md"), "# Testing").unwrap();
        std::fs::write(docs.join("orphan.md"), "# Orphan").unwrap();

        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![(
                "",
                vec![("project.context.md", "- [Testing](docs/testing.md)")],
            )],
        );

        let findings = check(&ctx_set);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].file.contains("orphan.md"));
        assert!(findings[0].message.contains("not referenced"));
    }

    #[test]
    fn doc_referenced_from_domain_context_is_clean() {
        let tmp = TempDir::new().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("billing.md"), "# Billing").unwrap();

        // Referenced from a nested domain context, not project.context.md
        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![(
                "src/billing",
                vec![(
                    "domain.context.md",
                    "See [billing docs](../../docs/billing.md)",
                )],
            )],
        );

        let findings = check(&ctx_set);
        assert!(findings.is_empty());
    }

    #[test]
    fn no_docs_directory_is_clean() {
        let tmp = TempDir::new().unwrap();
        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![("", vec![("project.context.md", "# Project")])],
        );

        let findings = check(&ctx_set);
        assert!(findings.is_empty());
    }

    #[test]
    fn nested_docs_are_checked() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("docs").join("guides");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("setup.md"), "# Setup").unwrap();

        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![(
                "",
                vec![("project.context.md", "# Project\n\nNo links here.")],
            )],
        );

        let findings = check(&ctx_set);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].file.contains("guides/setup.md"));
    }

    #[test]
    fn link_with_anchor_still_counts() {
        let tmp = TempDir::new().unwrap();
        let docs = tmp.path().join("docs");
        std::fs::create_dir_all(&docs).unwrap();
        std::fs::write(docs.join("auth.md"), "# Auth").unwrap();

        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![(
                "",
                vec![("project.context.md", "- [Tokens](docs/auth.md#tokens)")],
            )],
        );

        let findings = check(&ctx_set);
        assert!(findings.is_empty());
    }
}
