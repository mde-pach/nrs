use super::Finding;

const ROOT_CONTEXT_MAX: usize = 500;
const INNER_CONTEXT_MAX: usize = 300;

/// Check context file size limits. Warnings only — not errors.
pub fn check(relative_path: &str, filename: &str, content: &str) -> Vec<Finding> {
    let line_count = content.lines().count();

    let limit = match filename {
        "nrs.context.md" | "corporate.context.md" | "team.context.md" | "project.context.md" => {
            ROOT_CONTEXT_MAX
        }
        _ => INNER_CONTEXT_MAX,
    };

    if line_count > limit {
        vec![Finding::warning(
            relative_path,
            format!("{} lines (recommended max: {})", line_count, limit),
        )]
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::FindingKind;

    #[test]
    fn root_context_under_limit() {
        let content = "line\n".repeat(499);
        let findings = check("project.context.md", "project.context.md", &content);
        assert!(findings.is_empty());
    }

    #[test]
    fn root_context_over_limit_is_warning() {
        let content = "line\n".repeat(501);
        let findings = check("project.context.md", "project.context.md", &content);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Warning);
    }

    #[test]
    fn inner_context_over_limit_is_warning() {
        let content = "line\n".repeat(301);
        let findings = check("src/billing/domain.context.md", "domain.context.md", &content);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Warning);
    }

    #[test]
    fn nrs_context_uses_root_limit() {
        let content = "line\n".repeat(400);
        let findings = check("nrs.context.md", "nrs.context.md", &content);
        assert!(findings.is_empty());
    }
}
