use super::Finding;

/// Check reference rule violations.
///
/// Rules:
/// - domain.context.md must not reference higher layers (corporate, team terms)
/// - domain.context.md should not reference lower layers (implementation terms)
/// - implementation.context.md must not reference higher than its own layer
///
/// Detection is heuristic: looks for known layer-specific markers.
pub fn check(relative_path: &str, filename: &str, content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let content_lower = content.to_lowercase();

    match filename {
        "domain.context.md" => {
            // Domain must not reference implementation details
            for marker in IMPLEMENTATION_MARKERS {
                if content_lower.contains(marker) {
                    findings.push(Finding::error(
                        relative_path,
                        format!(
                            "domain context references implementation detail: '{}'",
                            marker
                        ),
                    ));
                }
            }
        }
        "implementation.context.md" => {
            // Implementation must not reference corporate/team layer concepts
            for marker in CORPORATE_MARKERS {
                if content_lower.contains(marker) {
                    findings.push(Finding::error(
                        relative_path,
                        format!(
                            "implementation context references corporate/team layer: '{}'",
                            marker
                        ),
                    ));
                }
            }
        }
        _ => {}
    }

    findings
}

/// Markers that suggest implementation-level content in a domain context.
/// These are type annotations, code patterns, and framework-specific terms
/// that shouldn't appear in business-oriented domain context.
const IMPLEMENTATION_MARKERS: &[&str] = &[
    ": number",
    ": string",
    ": boolean",
    "prisma",
    "repository",
    "endpoint",
    "api route",
    "http",
    "sql",
    "orm",
    "migration",
];

/// Markers that suggest corporate/team layer content in an implementation context.
const CORPORATE_MARKERS: &[&str] = &[
    "jira",
    "slack",
    "pagerduty",
    "oncall",
    "on-call",
    "deployment schedule",
    "pr review",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate::FindingKind;

    #[test]
    fn domain_context_with_prisma_is_error() {
        let content = "We use Prisma for data access";
        let findings = check("domain.context.md", "domain.context.md", content);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].kind, FindingKind::Error);
    }

    #[test]
    fn domain_context_with_type_annotation_is_error() {
        let content = "The price field is priceInCents: number";
        let findings = check("domain.context.md", "domain.context.md", content);
        assert!(!findings.is_empty());
    }

    #[test]
    fn domain_context_with_business_language_is_clean() {
        let content = "A product has a price and available stock.\nOrders can be cancelled only while pending.";
        let findings = check("domain.context.md", "domain.context.md", content);
        assert!(findings.is_empty());
    }

    #[test]
    fn implementation_context_with_jira_is_error() {
        let content = "Check the Jira ticket before implementing";
        let findings = check("implementation.context.md", "implementation.context.md", content);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].kind, FindingKind::Error);
    }

    #[test]
    fn implementation_context_with_patterns_is_clean() {
        let content = "Services are stateless functions.\nTyped domain errors thrown on failure.";
        let findings = check("implementation.context.md", "implementation.context.md", content);
        assert!(findings.is_empty());
    }

    #[test]
    fn other_context_files_are_not_checked() {
        let content = "We use Prisma and check Jira tickets";
        let findings = check("corporate.context.md", "corporate.context.md", content);
        assert!(findings.is_empty());
    }
}
