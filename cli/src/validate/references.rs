use crate::model::{ContextFile, Layer};

use super::{Finding, Validator, ValidatorScope};

pub struct ReferencesValidator;

impl Validator for ReferencesValidator {
    fn name(&self) -> &str {
        "references"
    }

    fn scope(&self) -> ValidatorScope {
        ValidatorScope::PerFile
    }

    fn check_file(&self, file: &ContextFile) -> Vec<Finding> {
        check(file)
    }
}

/// Check reference rule violations.
fn check(file: &ContextFile) -> Vec<Finding> {
    let mut findings = Vec::new();
    let content_lower = file.content.to_lowercase();

    match file.layer {
        Layer::Domain => {
            for marker in IMPLEMENTATION_MARKERS {
                if content_lower.contains(marker) {
                    findings.push(Finding::error(
                        &file.relative_path,
                        format!(
                            "domain context references implementation detail: '{}'",
                            marker
                        ),
                    ));
                }
            }
        }
        Layer::Implementation => {
            for marker in CORPORATE_MARKERS {
                if content_lower.contains(marker) {
                    findings.push(Finding::error(
                        &file.relative_path,
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

    fn file(name: &str, content: &str) -> ContextFile {
        ContextFile {
            relative_path: name.to_string(),
            filename: name.to_string(),
            layer: Layer::from_filename(name),
            content: content.to_string(),
        }
    }

    #[test]
    fn domain_context_with_prisma_is_error() {
        let f = file("domain.context.md", "We use Prisma for data access");
        let findings = check(&f);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].kind, FindingKind::Error);
    }

    #[test]
    fn domain_context_with_type_annotation_is_error() {
        let f = file(
            "domain.context.md",
            "The price field is priceInCents: number",
        );
        let findings = check(&f);
        assert!(!findings.is_empty());
    }

    #[test]
    fn domain_context_with_business_language_is_clean() {
        let f = file(
            "domain.context.md",
            "A product has a price and available stock.\nOrders can be cancelled only while pending.",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn implementation_context_with_jira_is_error() {
        let f = file(
            "implementation.context.md",
            "Check the Jira ticket before implementing",
        );
        let findings = check(&f);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].kind, FindingKind::Error);
    }

    #[test]
    fn implementation_context_with_patterns_is_clean() {
        let f = file(
            "implementation.context.md",
            "Services are stateless functions.\nTyped domain errors thrown on failure.",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn other_context_files_are_not_checked() {
        let f = file(
            "corporate.context.md",
            "We use Prisma and check Jira tickets",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }
}
