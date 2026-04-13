use crate::markdown;
use crate::model::{ContextFile, Layer};

use super::{Finding, Validator, ValidatorScope};

pub struct SourcePathsValidator;

impl Validator for SourcePathsValidator {
    fn name(&self) -> &str {
        "source_paths"
    }

    fn scope(&self) -> ValidatorScope {
        ValidatorScope::PerFile
    }

    fn check_file(&self, file: &ContextFile) -> Vec<Finding> {
        check(file)
    }
}

/// Detect source file path references in context files.
fn check(file: &ContextFile) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (i, line) in file.content.lines().enumerate() {
        let line_num = i + 1;

        // Skip lines that are just headings
        if line.trim_start().starts_with('#') {
            continue;
        }

        // Check backtick-wrapped paths (all layers except implementation)
        for segment in extract_path_like(line) {
            // Allow references to docs/
            if segment.starts_with("docs/") {
                continue;
            }
            // Allow references to other context files
            if segment.ends_with(".context.md") {
                continue;
            }

            findings.push(Finding::error(
                &file.relative_path,
                format!("line {}: source file path reference: {}", line_num, segment),
            ));
        }

        // Check markdown links [text](path) — allowed in implementation only
        if file.layer != Layer::Implementation {
            for link_target in markdown::extract_md_links(line) {
                // Skip external URLs
                if link_target.starts_with("http://") || link_target.starts_with("https://") {
                    continue;
                }
                // Skip anchors
                if link_target.starts_with('#') {
                    continue;
                }
                // Allow docs/ links
                if link_target.starts_with("docs/") {
                    continue;
                }
                // Allow context file links (handled by link rewriting)
                if link_target.ends_with(".context.md") {
                    continue;
                }

                if is_source_path(link_target) {
                    findings.push(Finding::error(
                        &file.relative_path,
                        format!(
                            "line {}: source file path reference: {}",
                            line_num, link_target
                        ),
                    ));
                }
            }
        }
    }

    findings
}

/// Extract path-like strings from a line of text.
fn extract_path_like(line: &str) -> Vec<&str> {
    let mut paths = Vec::new();

    // Match backtick-wrapped paths
    let mut rest = line;
    while let Some(start) = rest.find('`') {
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('`') {
            let candidate = &rest[..end];
            if is_source_path(candidate) {
                paths.push(candidate);
            }
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }

    paths
}

fn is_source_path(s: &str) -> bool {
    if !s.contains('/') && !s.contains('.') {
        return false;
    }

    let source_extensions = [
        ".ts", ".tsx", ".js", ".jsx", ".py", ".rs", ".go", ".java", ".rb", ".swift", ".kt", ".cs",
        ".cpp", ".c", ".h", ".vue", ".svelte",
    ];

    let source_prefixes = ["src/", "lib/", "app/", "pkg/", "cmd/", "internal/"];

    // Check if it looks like a file path with a source extension
    if source_extensions.iter().any(|ext| s.ends_with(ext)) && s.contains('/') {
        return true;
    }

    // Check if it starts with a known source directory
    if source_prefixes.iter().any(|prefix| s.starts_with(prefix)) {
        if !s.ends_with(".context.md") && !s.ends_with(".md") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Layer;
    use crate::validate::FindingKind;

    fn file(name: &str, content: &str) -> ContextFile {
        ContextFile {
            relative_path: name.to_string(),
            filename: name.to_string(),
            layer: Layer::from_filename(name),
            content: content.to_string(),
        }
    }

    #[test]
    fn detects_backtick_source_path() {
        let f = file(
            "domain.context.md",
            "See `src/services/product-service.ts` for details",
        );
        let findings = check(&f);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Error);
        assert!(findings[0]
            .message
            .contains("src/services/product-service.ts"));
    }

    #[test]
    fn allows_docs_links() {
        let f = file("project.context.md", "See `docs/testing.md` for test setup");
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn allows_context_file_references() {
        let f = file(
            "project.context.md",
            "See `domain.context.md` in the billing area",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn skips_headings() {
        let f = file(
            "domain.context.md",
            "# src/services/product-service.ts",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn detects_various_extensions() {
        for ext in &[".ts", ".py", ".rs", ".go", ".java", ".tsx", ".vue"] {
            let content = format!("See `src/foo/bar{}` here", ext);
            let f = file("domain.context.md", &content);
            let findings = check(&f);
            assert!(
                !findings.is_empty(),
                "should detect source path with extension {}",
                ext
            );
        }
    }

    #[test]
    fn detects_src_prefix_paths() {
        let f = file(
            "domain.context.md",
            "Look at `src/domains/billing/` for this",
        );
        let findings = check(&f);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn ignores_plain_text_without_backticks() {
        let f = file(
            "domain.context.md",
            "The services handle this via stateless functions",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn ignores_non_path_backtick_content() {
        let f = file("project.context.md", "Use `npm run dev` to start");
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    // ── Markdown link source path detection ───────────────────────────

    #[test]
    fn markdown_link_to_source_in_domain_is_error() {
        let f = file(
            "domain.context.md",
            "See [service](src/billing/service.ts) for details",
        );
        let findings = check(&f);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Error);
        assert!(findings[0].message.contains("src/billing/service.ts"));
    }

    #[test]
    fn markdown_link_to_source_in_project_is_error() {
        let f = file(
            "project.context.md",
            "See [handler](src/api/handler.go) for API",
        );
        let findings = check(&f);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn markdown_link_to_source_in_implementation_is_allowed() {
        let f = file(
            "implementation.context.md",
            "See [service](src/billing/service.ts) for details",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn markdown_link_to_docs_in_domain_is_allowed() {
        let f = file(
            "domain.context.md",
            "See [testing](docs/testing.md) for conventions",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn markdown_link_to_context_file_not_flagged_as_source() {
        let f = file(
            "project.context.md",
            "See [orders](src/orders/domain.context.md) for order rules",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn markdown_link_external_url_with_source_extension_allowed() {
        let f = file(
            "domain.context.md",
            "See [example](https://github.com/foo/bar.ts) for reference",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }
}
