/// Convert a markdown heading text to a GitHub-style anchor slug.
///
/// `"Domain Context — Orders"` → `"domain-context--orders"`
pub fn heading_to_anchor(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '-' {
                Some(c)
            } else if c == ' ' {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

/// Extract the first `# ` heading from content (H1 only).
pub fn first_heading(content: &str) -> Option<&str> {
    content
        .lines()
        .find(|line| line.starts_with("# "))
        .map(|line| line[2..].trim())
}

/// Rewrite markdown links targeting `*.context.md` to point to the compiled
/// output file. Same-directory links become anchors; cross-directory links
/// become `path/CLAUDE.md#anchor`.
pub fn rewrite_context_links(
    content: &str,
    dir_files: &[crate::model::ContextFile],
    all: &crate::model::ContextSet,
    current_dir: &std::path::Path,
    output_filename: &str,
) -> String {
    let mut result = String::with_capacity(content.len());
    let mut rest = content;

    while let Some(link_start) = rest.find("](") {
        // Copy everything up to and including ](
        result.push_str(&rest[..link_start + 2]);
        rest = &rest[link_start + 2..];

        if let Some(paren_end) = rest.find(')') {
            let target = &rest[..paren_end];

            if target.ends_with(".context.md") && !target.starts_with("http") {
                let rewritten = rewrite_single_context_link(
                    target,
                    dir_files,
                    all,
                    current_dir,
                    output_filename,
                );
                result.push_str(&rewritten);
            } else {
                result.push_str(target);
            }

            result.push(')');
            rest = &rest[paren_end + 1..];
        } else {
            // Malformed link — copy as-is
            result.push_str(rest);
            rest = "";
        }
    }

    result.push_str(rest);
    result
}

/// Normalize a path by resolving `.` and `..` components without touching
/// the filesystem (no canonicalize/symlink resolution).
fn normalize_path(path: &std::path::Path) -> std::path::PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            other => components.push(other),
        }
    }
    components.iter().collect()
}

fn rewrite_single_context_link(
    target: &str,
    dir_files: &[crate::model::ContextFile],
    all: &crate::model::ContextSet,
    current_dir: &std::path::Path,
    output_filename: &str,
) -> String {
    // Split target into directory part and filename
    let (dir_part, filename) = match target.rfind('/') {
        Some(pos) => (&target[..=pos], &target[pos + 1..]),
        None => ("", target),
    };

    let is_same_dir = dir_part.is_empty();

    // Find the target file to get its first heading
    let anchor = if is_same_dir {
        // Look in same-directory files
        dir_files
            .iter()
            .find(|f| f.filename == filename)
            .and_then(|f| first_heading(&f.content))
            .map(heading_to_anchor)
    } else {
        // Resolve the target relative to current_dir, normalize .., then
        // strip root to get the ContextFile relative_path.
        let abs_target = normalize_path(&current_dir.join(target));
        let rel_target = abs_target
            .strip_prefix(&all.root)
            .ok()
            .map(|p| p.to_string_lossy().to_string());

        rel_target.and_then(|rel| {
            all.all_files()
                .find(|f| f.relative_path == rel)
                .and_then(|f| first_heading(&f.content))
                .map(heading_to_anchor)
        })
    };

    let anchor_suffix = anchor
        .map(|a| format!("#{}", a))
        .unwrap_or_default();

    if is_same_dir {
        // Content is already inline in same CLAUDE.md — anchor only
        format!("{}{}", output_filename, anchor_suffix)
    } else {
        // Point to the target directory's compiled output
        format!("{}{}{}", dir_part, output_filename, anchor_suffix)
    }
}

/// Extract markdown link targets from a line: `[text](target)` → `target`.
pub fn extract_md_links(line: &str) -> Vec<&str> {
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
    use crate::model::{ContextFile, ContextSet, DirectoryContext, Layer};
    use std::path::PathBuf;

    #[test]
    fn extracts_single_link() {
        assert_eq!(extract_md_links("[text](target)"), vec!["target"]);
    }

    #[test]
    fn extracts_multiple_links() {
        assert_eq!(
            extract_md_links("- [A](foo.md) and [B](bar.md)"),
            vec!["foo.md", "bar.md"]
        );
    }

    #[test]
    fn no_links() {
        assert!(extract_md_links("no links here").is_empty());
    }

    #[test]
    fn empty_target_skipped() {
        assert!(extract_md_links("[text]()").is_empty());
    }

    // ── heading_to_anchor ─────────────────────────────────────────────

    #[test]
    fn anchor_simple_word() {
        assert_eq!(heading_to_anchor("Project"), "project");
    }

    #[test]
    fn anchor_with_spaces() {
        assert_eq!(heading_to_anchor("Domain Context"), "domain-context");
    }

    #[test]
    fn anchor_with_em_dash() {
        assert_eq!(
            heading_to_anchor("Domain Context — Orders"),
            "domain-context--orders"
        );
    }

    #[test]
    fn anchor_strips_special_chars() {
        assert_eq!(
            heading_to_anchor("Hello, World! (test)"),
            "hello-world-test"
        );
    }

    #[test]
    fn anchor_preserves_hyphens() {
        assert_eq!(heading_to_anchor("e-commerce"), "e-commerce");
    }

    #[test]
    fn anchor_preserves_numbers() {
        assert_eq!(heading_to_anchor("Layer 5"), "layer-5");
    }

    // ── first_heading ─────────────────────────────────────────────────

    #[test]
    fn first_heading_extracts_h1() {
        assert_eq!(
            first_heading("# My Heading\n\nSome content"),
            Some("My Heading")
        );
    }

    #[test]
    fn first_heading_picks_first_h1_only() {
        assert_eq!(
            first_heading("## Not H1\n# Actual H1\n# Second H1"),
            Some("Actual H1")
        );
    }

    #[test]
    fn first_heading_none_when_absent() {
        assert_eq!(first_heading("No headings here\n## Only H2"), None);
    }

    #[test]
    fn first_heading_trims_whitespace() {
        assert_eq!(first_heading("#   Padded   "), Some("Padded"));
    }

    // ── rewrite_context_links ─────────────────────────────────────────

    fn make_ctx_set(dirs: Vec<(&str, Vec<(&str, &str)>)>) -> (ContextSet, PathBuf) {
        let root = PathBuf::from("/project");
        let directories = dirs
            .into_iter()
            .map(|(rel_dir, files)| {
                let dir = if rel_dir.is_empty() {
                    root.clone()
                } else {
                    root.join(rel_dir)
                };
                let ctx_files = files
                    .into_iter()
                    .map(|(name, content)| {
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
                    dir,
                    relative_dir: rel_dir.to_string(),
                    files: ctx_files,
                }
            })
            .collect();
        (ContextSet { root: root.clone(), directories }, root)
    }

    #[test]
    fn rewrite_same_dir_context_link_to_anchor() {
        let (ctx_set, root) = make_ctx_set(vec![(
            "",
            vec![
                ("domain.context.md", "# Domain Context\n\nBusiness rules."),
                (
                    "implementation.context.md",
                    "# Implementation Context\n\nPatterns.",
                ),
            ],
        )]);
        let dir_files = &ctx_set.directories[0].files;

        let input = "See [Impl](implementation.context.md) for patterns.";
        let result = rewrite_context_links(input, dir_files, &ctx_set, &root, "CLAUDE.md");

        assert_eq!(
            result,
            "See [Impl](CLAUDE.md#implementation-context) for patterns."
        );
    }

    #[test]
    fn rewrite_cross_dir_context_link() {
        let (ctx_set, root) = make_ctx_set(vec![
            ("", vec![("project.context.md", "# Project\n\nMap.")]),
            (
                "src/orders",
                vec![("domain.context.md", "# Domain Context — Orders\n\nOrder rules.")],
            ),
        ]);
        let dir_files = &ctx_set.directories[0].files;

        let input = "See [Orders](src/orders/domain.context.md) for order rules.";
        let result = rewrite_context_links(input, dir_files, &ctx_set, &root, "CLAUDE.md");

        assert_eq!(
            result,
            "See [Orders](src/orders/CLAUDE.md#domain-context--orders) for order rules."
        );
    }

    #[test]
    fn rewrite_leaves_non_context_links_unchanged() {
        let (ctx_set, root) = make_ctx_set(vec![(
            "",
            vec![("project.context.md", "# Project")],
        )]);
        let dir_files = &ctx_set.directories[0].files;

        let input = "See [Testing](docs/testing.md) for test setup.";
        let result = rewrite_context_links(input, dir_files, &ctx_set, &root, "CLAUDE.md");

        assert_eq!(result, input);
    }

    #[test]
    fn rewrite_leaves_external_urls_unchanged() {
        let (ctx_set, root) = make_ctx_set(vec![(
            "",
            vec![("project.context.md", "# Project")],
        )]);
        let dir_files = &ctx_set.directories[0].files;

        let input = "See [Docs](https://example.com/domain.context.md) for info.";
        let result = rewrite_context_links(input, dir_files, &ctx_set, &root, "CLAUDE.md");

        assert_eq!(result, input);
    }

    #[test]
    fn rewrite_multiple_links_in_one_line() {
        let (ctx_set, root) = make_ctx_set(vec![(
            "",
            vec![
                ("domain.context.md", "# Domain Context\n\nRules."),
                (
                    "implementation.context.md",
                    "# Implementation Context\n\nPatterns.",
                ),
            ],
        )]);
        let dir_files = &ctx_set.directories[0].files;

        let input =
            "- [Domain](domain.context.md) and [Impl](implementation.context.md)";
        let result = rewrite_context_links(input, dir_files, &ctx_set, &root, "CLAUDE.md");

        assert_eq!(
            result,
            "- [Domain](CLAUDE.md#domain-context) and [Impl](CLAUDE.md#implementation-context)"
        );
    }

    #[test]
    fn rewrite_fallback_without_heading() {
        let (ctx_set, root) = make_ctx_set(vec![(
            "",
            vec![("domain.context.md", "No heading here, just text.")],
        )]);
        let dir_files = &ctx_set.directories[0].files;

        let input = "See [Domain](domain.context.md)";
        let result = rewrite_context_links(input, dir_files, &ctx_set, &root, "CLAUDE.md");

        // No anchor available — just point to the file
        assert_eq!(result, "See [Domain](CLAUDE.md)");
    }

    #[test]
    fn rewrite_cross_dir_with_dotdot_path() {
        let (ctx_set, _root) = make_ctx_set(vec![
            (
                "src/billing",
                vec![("domain.context.md", "# Domain Context — Billing\n\nBilling.")],
            ),
            (
                "src/orders",
                vec![("domain.context.md", "# Domain Context — Orders\n\nOrders.")],
            ),
        ]);
        let dir_files = &ctx_set.directories[1].files; // orders
        let orders_dir = &ctx_set.directories[1].dir;

        let input = "See [Billing](../billing/domain.context.md) for billing rules.";
        let result =
            rewrite_context_links(input, dir_files, &ctx_set, orders_dir, "CLAUDE.md");

        assert_eq!(
            result,
            "See [Billing](../billing/CLAUDE.md#domain-context--billing) for billing rules."
        );
    }
}
