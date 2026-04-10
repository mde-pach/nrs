use super::Finding;

/// Detect source file path references in context files.
/// Looks for patterns like src/, .ts, .tsx, .js, .py, .rs etc. in path-like strings.
/// Allows references to other *.context.md and docs/ links.
pub fn check(relative_path: &str, content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;

        // Skip markdown links to docs/ (allowed by spec)
        // Skip markdown links to *.context.md (allowed)
        // Skip lines that are just headings
        if line.trim_start().starts_with('#') {
            continue;
        }

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
                relative_path,
                format!("line {}: source file path reference: {}", line_num, segment),
            ));
        }
    }

    findings
}

/// Extract path-like strings from a line of text.
/// A path-like string contains a `/` and ends with a known source extension,
/// or starts with src/, lib/, app/, etc.
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
        ".ts", ".tsx", ".js", ".jsx", ".py", ".rs", ".go", ".java", ".rb", ".swift", ".kt",
        ".cs", ".cpp", ".c", ".h", ".vue", ".svelte",
    ];

    let source_prefixes = ["src/", "lib/", "app/", "pkg/", "cmd/", "internal/"];

    // Check if it looks like a file path with a source extension
    if source_extensions.iter().any(|ext| s.ends_with(ext)) && s.contains('/') {
        return true;
    }

    // Check if it starts with a known source directory
    if source_prefixes.iter().any(|prefix| s.starts_with(prefix)) {
        // But not if it's a context file or doc
        if !s.ends_with(".context.md") && !s.ends_with(".md") {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::FindingKind;

    #[test]
    fn detects_backtick_source_path() {
        let content = "See `src/services/product-service.ts` for details";
        let findings = check("domain.context.md", content);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Error);
        assert!(findings[0].message.contains("src/services/product-service.ts"));
    }

    #[test]
    fn allows_docs_links() {
        let content = "See `docs/testing.md` for test setup";
        let findings = check("project.context.md", content);
        assert!(findings.is_empty());
    }

    #[test]
    fn allows_context_file_references() {
        let content = "See `domain.context.md` in the billing area";
        let findings = check("project.context.md", content);
        assert!(findings.is_empty());
    }

    #[test]
    fn skips_headings() {
        let content = "# src/services/product-service.ts";
        let findings = check("domain.context.md", content);
        assert!(findings.is_empty());
    }

    #[test]
    fn detects_various_extensions() {
        for ext in &[".ts", ".py", ".rs", ".go", ".java", ".tsx", ".vue"] {
            let content = format!("See `src/foo/bar{}` here", ext);
            let findings = check("domain.context.md", &content);
            assert!(
                !findings.is_empty(),
                "should detect source path with extension {}",
                ext
            );
        }
    }

    #[test]
    fn detects_src_prefix_paths() {
        let content = "Look at `src/domains/billing/` for this";
        let findings = check("domain.context.md", content);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn ignores_plain_text_without_backticks() {
        let content = "The services handle this via stateless functions";
        let findings = check("domain.context.md", content);
        assert!(findings.is_empty());
    }

    #[test]
    fn ignores_non_path_backtick_content() {
        let content = "Use `npm run dev` to start";
        let findings = check("project.context.md", content);
        assert!(findings.is_empty());
    }
}
