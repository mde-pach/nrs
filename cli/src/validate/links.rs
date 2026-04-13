use crate::markdown;
use crate::model::DirectoryContext;

use super::{Finding, Validator, ValidatorScope};
use std::path::Path;

pub struct LinksValidator;

impl Validator for LinksValidator {
    fn name(&self) -> &str {
        "links"
    }

    fn scope(&self) -> ValidatorScope {
        ValidatorScope::PerDirectory
    }

    fn check_directory(&self, root: &Path, dir_ctx: &DirectoryContext) -> Vec<Finding> {
        check(root, dir_ctx)
    }
}

/// Check that markdown links in context files resolve to existing files.
fn check(root: &Path, dir_ctx: &DirectoryContext) -> Vec<Finding> {
    let mut findings = Vec::new();

    for file in &dir_ctx.files {
        for (i, line) in file.content.lines().enumerate() {
            let line_num = i + 1;

            for link_target in markdown::extract_md_links(line) {
                if link_target.starts_with("http://") || link_target.starts_with("https://") {
                    continue;
                }
                if link_target.starts_with('#') {
                    continue;
                }
                // Context file links are valid — they get rewritten by generate.
                if link_target.ends_with(".context.md") {
                    continue;
                }
                // Only validate documentation links (.md files). Source file
                // links are the source_paths validator's concern.
                if !link_target.ends_with(".md") {
                    continue;
                }

                let target_path = dir_ctx.dir.join(link_target);
                if !target_path.exists() {
                    let from_root = root.join(link_target);
                    if !from_root.exists() {
                        findings.push(Finding::error(
                            &file.relative_path,
                            format!("line {}: broken link: {}", line_num, link_target),
                        ));
                    }
                }
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ContextFile, Layer};
    use crate::validate::FindingKind;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_dir_ctx(dir: PathBuf, files: Vec<(&str, &str)>) -> DirectoryContext {
        let ctx_files = files
            .into_iter()
            .map(|(name, content)| ContextFile {
                relative_path: name.to_string(),
                filename: name.to_string(),
                layer: Layer::from_filename(name),
                content: content.to_string(),
            })
            .collect();
        DirectoryContext {
            dir,
            relative_dir: String::new(),
            files: ctx_files,
        }
    }

    #[test]
    fn valid_link_passes() {
        let tmp = TempDir::new().unwrap();
        let docs_dir = tmp.path().join("docs");
        std::fs::create_dir_all(&docs_dir).unwrap();
        std::fs::write(docs_dir.join("testing.md"), "# Testing").unwrap();

        let dir_ctx = make_dir_ctx(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "- [Testing](docs/testing.md)")],
        );

        let findings = check(tmp.path(), &dir_ctx);
        assert!(findings.is_empty());
    }

    #[test]
    fn broken_link_is_error() {
        let tmp = TempDir::new().unwrap();

        let dir_ctx = make_dir_ctx(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "- [Testing](docs/testing.md)")],
        );

        let findings = check(tmp.path(), &dir_ctx);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Error);
        assert!(findings[0].message.contains("docs/testing.md"));
    }

    #[test]
    fn external_urls_skipped() {
        let tmp = TempDir::new().unwrap();

        let dir_ctx = make_dir_ctx(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "- [Docs](https://example.com)")],
        );

        let findings = check(tmp.path(), &dir_ctx);
        assert!(findings.is_empty());
    }

    #[test]
    fn anchors_skipped() {
        let tmp = TempDir::new().unwrap();

        let dir_ctx = make_dir_ctx(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "- [Section](#architecture)")],
        );

        let findings = check(tmp.path(), &dir_ctx);
        assert!(findings.is_empty());
    }

    #[test]
    fn checks_domain_context_links() {
        let tmp = TempDir::new().unwrap();

        let dir_ctx = make_dir_ctx(
            tmp.path().to_path_buf(),
            vec![("domain.context.md", "- [Broken](nonexistent.md)")],
        );

        let findings = check(tmp.path(), &dir_ctx);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Error);
        assert!(findings[0].message.contains("nonexistent.md"));
    }

    #[test]
    fn checks_implementation_context_links() {
        let tmp = TempDir::new().unwrap();

        let dir_ctx = make_dir_ctx(
            tmp.path().to_path_buf(),
            vec![("implementation.context.md", "- [Missing](docs/arch.md)")],
        );

        let findings = check(tmp.path(), &dir_ctx);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("docs/arch.md"));
    }

    #[test]
    fn valid_link_in_domain_context_passes() {
        let tmp = TempDir::new().unwrap();
        let docs_dir = tmp.path().join("docs");
        std::fs::create_dir_all(&docs_dir).unwrap();
        std::fs::write(docs_dir.join("billing.md"), "# Billing").unwrap();

        let dir_ctx = make_dir_ctx(
            tmp.path().to_path_buf(),
            vec![("domain.context.md", "- [Billing](docs/billing.md)")],
        );

        let findings = check(tmp.path(), &dir_ctx);
        assert!(findings.is_empty());
    }

    #[test]
    fn checks_custom_context_links() {
        let tmp = TempDir::new().unwrap();

        let dir_ctx = make_dir_ctx(
            tmp.path().to_path_buf(),
            vec![("custom.context.md", "- [Gone](gone.md)")],
        );

        let findings = check(tmp.path(), &dir_ctx);
        assert_eq!(findings.len(), 1);
    }
}
