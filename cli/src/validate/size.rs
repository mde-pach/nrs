use crate::model::ContextFile;

use super::{Finding, Validator, ValidatorScope};

pub struct SizeValidator;

impl Validator for SizeValidator {
    fn name(&self) -> &str {
        "size"
    }

    fn scope(&self) -> ValidatorScope {
        ValidatorScope::PerFile
    }

    fn check_file(&self, file: &ContextFile) -> Vec<Finding> {
        check(file)
    }
}

/// Check context file size limits. Warnings only — not errors.
fn check(file: &ContextFile) -> Vec<Finding> {
    let line_count = file.content.lines().count();
    let limit = file.layer.size_limit();

    if line_count > limit {
        vec![Finding::warning(
            &file.relative_path,
            format!("{} lines (recommended max: {})", line_count, limit),
        )]
    } else {
        vec![]
    }
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
    fn root_context_under_limit() {
        let content = "line\n".repeat(499);
        let findings = check(&file("project.context.md", &content));
        assert!(findings.is_empty());
    }

    #[test]
    fn root_context_over_limit_is_warning() {
        let content = "line\n".repeat(501);
        let findings = check(&file("project.context.md", &content));
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Warning);
    }

    #[test]
    fn inner_context_over_limit_is_warning() {
        let content = "line\n".repeat(301);
        let f = ContextFile {
            relative_path: "src/billing/domain.context.md".to_string(),
            filename: "domain.context.md".to_string(),
            layer: Layer::Domain,
            content,
        };
        let findings = check(&f);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Warning);
    }

    #[test]
    fn nrs_context_uses_root_limit() {
        let content = "line\n".repeat(400);
        let findings = check(&file("nrs.context.md", &content));
        assert!(findings.is_empty());
    }
}
