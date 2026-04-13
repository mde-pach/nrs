use std::collections::HashSet;

use crate::model::ContextSet;
use crate::validate::{Finding, Validator, ValidatorScope};

pub struct DuplicationValidator;

impl Validator for DuplicationValidator {
    fn name(&self) -> &str {
        "duplication"
    }

    fn scope(&self) -> ValidatorScope {
        ValidatorScope::Global
    }

    fn check_all(&self, ctx_set: &ContextSet) -> anyhow::Result<Vec<Finding>> {
        Ok(check(ctx_set))
    }
}

/// Minimum number of consecutive matching lines to count as a block.
const MIN_BLOCK_LINES: usize = 3;

/// Jaccard similarity threshold above which two blocks are considered
/// near-duplicates.  0.7 catches meaningful paraphrasing while avoiding
/// false positives from generic markdown prose.
const SIMILARITY_THRESHOLD: f64 = 0.7;

/// Detect duplicated content blocks across context files.
///
/// Two detection modes:
/// 1. **Exact** — sliding window of `MIN_BLOCK_LINES` normalised lines that
///    match verbatim across files (catches copy-paste).
/// 2. **Similar** — word-level Jaccard similarity on the same windows.  When
///    the similarity exceeds `SIMILARITY_THRESHOLD` the block is flagged as a
///    near-duplicate (catches paraphrased content).
pub fn check(ctx_set: &ContextSet) -> Vec<Finding> {
    // Collect (relative_path, normalised_lines) for every context file.
    let files: Vec<(&str, Vec<String>)> = ctx_set
        .all_files()
        .map(|f| (f.relative_path.as_str(), normalise(&f.content)))
        .collect();

    if files.len() < 2 {
        return Vec::new();
    }

    // Pre-compute word sets for every window in every file.
    let file_windows: Vec<(&str, Vec<WindowInfo>)> = files
        .iter()
        .map(|(path, lines)| {
            let windows = if lines.len() >= MIN_BLOCK_LINES {
                lines
                    .windows(MIN_BLOCK_LINES)
                    .map(|w| WindowInfo {
                        lines: w.to_vec(),
                        words: word_set(w),
                    })
                    .collect()
            } else {
                Vec::new()
            };
            (*path, windows)
        })
        .collect();

    let mut findings = Vec::new();
    // Track which (owner, duplicate) pairs we already reported.
    let mut reported: HashSet<(usize, usize)> = HashSet::new();

    // Compare every pair of files.
    for i in 0..file_windows.len() {
        for j in (i + 1)..file_windows.len() {
            if reported.contains(&(i, j)) {
                continue;
            }

            let (path_a, wins_a) = &file_windows[i];
            let (path_b, wins_b) = &file_windows[j];

            for wa in wins_a {
                for wb in wins_b {
                    // Exact match
                    if wa.lines == wb.lines {
                        reported.insert((i, j));
                        let preview = first_meaningful_line(&wa.lines);
                        findings.push(Finding::warning(
                            path_b.to_string(),
                            format!(
                                "duplicated content block also found in {} (near \"{}\")",
                                path_a, preview
                            ),
                        ));
                        break;
                    }

                    // Similarity match
                    let sim = jaccard(&wa.words, &wb.words);
                    if sim >= SIMILARITY_THRESHOLD {
                        reported.insert((i, j));
                        let preview = first_meaningful_line(&wa.lines);
                        findings.push(Finding::warning(
                            path_b.to_string(),
                            format!(
                                "near-duplicate content ({:.0}% similar) also found in {} (near \"{}\")",
                                sim * 100.0,
                                path_a,
                                preview
                            ),
                        ));
                        break;
                    }
                }
                // Stop comparing windows once we've flagged this pair.
                if reported.contains(&(i, j)) {
                    break;
                }
            }
        }
    }

    findings
}

struct WindowInfo {
    lines: Vec<String>,
    words: HashSet<String>,
}

/// Extract the set of distinct words from a block of normalised lines.
fn word_set(lines: &[String]) -> HashSet<String> {
    lines
        .iter()
        .flat_map(|l| {
            l.split(|c: char| !c.is_alphanumeric())
                .filter(|w| w.len() > 2) // drop short noise words
                .map(|w| w.to_string())
        })
        .collect()
}

/// Jaccard similarity: |A ∩ B| / |A ∪ B|.
fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    intersection / union
}

/// Normalise file content into comparable lines.
///
/// Strips markdown structure (headings, blank lines, horizontal rules,
/// table separators) so only meaningful prose/data lines remain.
fn normalise(content: &str) -> Vec<String> {
    content
        .lines()
        .map(|l| l.trim())
        .filter(|l| {
            !l.is_empty()
                && !l.starts_with('#')
                && !l.starts_with("---")
                && !l.starts_with("___")
                && !l.chars().all(|c| c == '-' || c == '|' || c == ' ')
        })
        .map(|l| l.to_lowercase())
        .collect()
}

/// Pick the first non-trivial line from a block for the warning message.
fn first_meaningful_line(block: &[String]) -> String {
    let line = block.first().map(|s| s.as_str()).unwrap_or("");
    if line.len() > 80 {
        format!("{}…", &line[..77])
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ContextFile, DirectoryContext, Layer};
    use tempfile::TempDir;

    fn make_ctx_set(tmp: &std::path::Path, dirs: Vec<(&str, Vec<(&str, &str)>)>) -> ContextSet {
        let directories = dirs
            .into_iter()
            .map(|(rel_dir, file_entries)| {
                let d = if rel_dir.is_empty() {
                    tmp.to_path_buf()
                } else {
                    tmp.join(rel_dir)
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
            root: tmp.to_path_buf(),
            directories,
        }
    }

    #[test]
    fn detects_duplicated_block_across_files() {
        let tmp = TempDir::new().unwrap();
        let shared = "Orders are created when a customer checks out.\n\
                       Each order captures a snapshot of product prices.\n\
                       Orders cannot be modified after payment confirmation.\n";
        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![
                (
                    "",
                    vec![("project.context.md", &format!("# Project\n\n{}", shared))],
                ),
                (
                    "src/orders",
                    vec![("domain.context.md", &format!("# Orders\n\n{}", shared))],
                ),
            ],
        );

        let findings = check(&ctx_set);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("duplicated content block"));
        assert!(findings[0].message.contains("project.context.md"));
    }

    #[test]
    fn detects_near_duplicate_paraphrased_content() {
        let tmp = TempDir::new().unwrap();
        let original = "Orders are created when a customer checks out.\n\
                         Each order captures a snapshot of product prices.\n\
                         Orders cannot be modified after payment confirmation.\n";
        let paraphrased = "Orders are created when the customer checks out.\n\
                            Each order captures a snapshot of the product prices.\n\
                            Orders cannot be changed after payment confirmation.\n";
        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![
                (
                    "",
                    vec![(
                        "project.context.md",
                        &format!("# Project\n\n{}", original),
                    )],
                ),
                (
                    "src/orders",
                    vec![(
                        "domain.context.md",
                        &format!("# Orders\n\n{}", paraphrased),
                    )],
                ),
            ],
        );

        let findings = check(&ctx_set);
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("near-duplicate content"));
        assert!(findings[0].message.contains("similar"));
    }

    #[test]
    fn no_warning_for_unique_content() {
        let tmp = TempDir::new().unwrap();
        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![
                (
                    "",
                    vec![(
                        "project.context.md",
                        "# Project\n\nThis is about the project.\nIt has unique content.\nNothing shared.",
                    )],
                ),
                (
                    "src/billing",
                    vec![(
                        "domain.context.md",
                        "# Billing\n\nBilling handles payments.\nInvoices are generated monthly.\nTax rules apply.",
                    )],
                ),
            ],
        );

        let findings = check(&ctx_set);
        assert!(findings.is_empty());
    }

    #[test]
    fn ignores_shared_headings_and_structure() {
        let tmp = TempDir::new().unwrap();
        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![
                (
                    "",
                    vec![(
                        "project.context.md",
                        "# Overview\n\n## Rules\n\nProject-specific rules here.\nAnother line.\nThird line.",
                    )],
                ),
                (
                    "src/auth",
                    vec![(
                        "domain.context.md",
                        "# Overview\n\n## Rules\n\nAuth-specific rules here.\nDifferent line.\nAnother third.",
                    )],
                ),
            ],
        );

        let findings = check(&ctx_set);
        assert!(findings.is_empty());
    }

    #[test]
    fn no_findings_for_single_file() {
        let tmp = TempDir::new().unwrap();
        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![(
                "",
                vec![(
                    "project.context.md",
                    "# Project\n\nSome content.\nMore content.\nEven more.",
                )],
            )],
        );

        let findings = check(&ctx_set);
        assert!(findings.is_empty());
    }

    #[test]
    fn short_files_below_threshold_skipped() {
        let tmp = TempDir::new().unwrap();
        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![
                ("", vec![("project.context.md", "# Project\n\nShort.")]),
                ("src/x", vec![("domain.context.md", "# Domain\n\nShort.")]),
            ],
        );

        let findings = check(&ctx_set);
        assert!(findings.is_empty());
    }

    #[test]
    fn completely_different_topics_not_flagged() {
        let tmp = TempDir::new().unwrap();
        let ctx_set = make_ctx_set(
            tmp.path(),
            vec![
                (
                    "",
                    vec![(
                        "project.context.md",
                        "# Project\n\n\
                         Authentication uses JWT tokens with RS256 signing.\n\
                         Tokens expire after 30 minutes of inactivity.\n\
                         Refresh tokens are stored in HTTP-only cookies.\n",
                    )],
                ),
                (
                    "src/billing",
                    vec![(
                        "domain.context.md",
                        "# Billing\n\n\
                         Invoices are generated on the first of each month.\n\
                         Payment processing uses Stripe as the gateway.\n\
                         Failed payments trigger a three-retry backoff sequence.\n",
                    )],
                ),
            ],
        );

        let findings = check(&ctx_set);
        assert!(findings.is_empty());
    }
}
