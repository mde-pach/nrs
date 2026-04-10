use super::Finding;
use std::collections::BTreeMap;
use std::path::Path;

/// Check that markdown links in project.context.md (the map) resolve to existing files.
pub fn check(
    root: &Path,
    dir: &Path,
    relative_dir: &str,
    files: &BTreeMap<String, String>,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    let Some(content) = files.get("project.context.md") else {
        return findings;
    };

    let relative_path = if relative_dir.is_empty() {
        "project.context.md".to_string()
    } else {
        format!("{}/project.context.md", relative_dir)
    };

    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;

        for link_target in extract_md_links(line) {
            // Skip external URLs
            if link_target.starts_with("http://") || link_target.starts_with("https://") {
                continue;
            }
            // Skip anchors
            if link_target.starts_with('#') {
                continue;
            }

            let target_path = dir.join(link_target);
            if !target_path.exists() {
                // Also check relative to root
                let from_root = root.join(link_target);
                if !from_root.exists() {
                    findings.push(Finding::error(
                        &relative_path,
                        format!("line {}: broken link: {}", line_num, link_target),
                    ));
                }
            }
        }
    }

    findings
}

/// Extract markdown link targets: \[text\](target)
fn extract_md_links(line: &str) -> Vec<&str> {
    let mut links = Vec::new();
    let mut rest = line;

    while let Some(pos) = rest.find("](") {
        rest = &rest[pos + 2..];
        if let Some(end) = rest.find(')') {
            let target = &rest[..end];
            if !target.is_empty() {
                links.push(target);
            }
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }

    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::FindingKind;
    use tempfile::TempDir;

    #[test]
    fn valid_link_passes() {
        let tmp = TempDir::new().unwrap();
        let docs_dir = tmp.path().join("docs");
        std::fs::create_dir_all(&docs_dir).unwrap();
        std::fs::write(docs_dir.join("testing.md"), "# Testing").unwrap();

        let mut files = BTreeMap::new();
        files.insert(
            "project.context.md".to_string(),
            "- [Testing](docs/testing.md)".to_string(),
        );

        let findings = check(tmp.path(), tmp.path(), "", &files);
        assert!(findings.is_empty());
    }

    #[test]
    fn broken_link_is_error() {
        let tmp = TempDir::new().unwrap();

        let mut files = BTreeMap::new();
        files.insert(
            "project.context.md".to_string(),
            "- [Testing](docs/testing.md)".to_string(),
        );

        let findings = check(tmp.path(), tmp.path(), "", &files);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Error);
        assert!(findings[0].message.contains("docs/testing.md"));
    }

    #[test]
    fn external_urls_skipped() {
        let tmp = TempDir::new().unwrap();

        let mut files = BTreeMap::new();
        files.insert(
            "project.context.md".to_string(),
            "- [Docs](https://example.com)".to_string(),
        );

        let findings = check(tmp.path(), tmp.path(), "", &files);
        assert!(findings.is_empty());
    }

    #[test]
    fn anchors_skipped() {
        let tmp = TempDir::new().unwrap();

        let mut files = BTreeMap::new();
        files.insert(
            "project.context.md".to_string(),
            "- [Section](#architecture)".to_string(),
        );

        let findings = check(tmp.path(), tmp.path(), "", &files);
        assert!(findings.is_empty());
    }

    #[test]
    fn only_checks_project_context() {
        let tmp = TempDir::new().unwrap();

        let mut files = BTreeMap::new();
        files.insert(
            "domain.context.md".to_string(),
            "- [Broken](nonexistent.md)".to_string(),
        );

        let findings = check(tmp.path(), tmp.path(), "", &files);
        assert!(findings.is_empty());
    }

    #[test]
    fn extract_md_links_works() {
        assert_eq!(
            extract_md_links("- [A](foo.md) and [B](bar.md)"),
            vec!["foo.md", "bar.md"]
        );
        assert!(extract_md_links("no links here").is_empty());
        assert_eq!(extract_md_links("[text](target)"), vec!["target"]);
    }
}
