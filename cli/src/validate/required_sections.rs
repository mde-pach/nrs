use crate::model::{ContextFile, Layer};

use super::{Finding, Validator, ValidatorScope};

pub struct RequiredSectionsValidator;

impl Validator for RequiredSectionsValidator {
    fn name(&self) -> &str {
        "required_sections"
    }

    fn scope(&self) -> ValidatorScope {
        ValidatorScope::PerFile
    }

    fn check_file(&self, file: &ContextFile) -> Vec<Finding> {
        check(file)
    }
}

fn check(file: &ContextFile) -> Vec<Finding> {
    if file.layer != Layer::Project {
        return vec![];
    }

    let has_commands = file
        .content
        .lines()
        .any(|line| line.starts_with("## Commands"));

    if !has_commands {
        vec![Finding::error(
            &file.relative_path,
            "project.context.md must contain a `## Commands` section",
        )]
    } else {
        vec![]
    }
}

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
    fn project_context_with_commands_section_passes() {
        let f = file(
            "project.context.md",
            "# Project\n\n## Commands\n\n- `npm test` — run tests\n",
        );
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn project_context_without_commands_section_is_error() {
        let f = file(
            "project.context.md",
            "# Project\n\nSome description of the project.\n",
        );
        let findings = check(&f);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Error);
    }

    #[test]
    fn other_layers_are_skipped() {
        let f = file("domain.context.md", "# Domain\n\nNo commands here.\n");
        let findings = check(&f);
        assert!(findings.is_empty());
    }

    #[test]
    fn commands_as_h3_does_not_count() {
        let f = file(
            "project.context.md",
            "# Project\n\n### Commands\n\n- `npm test`\n",
        );
        let findings = check(&f);
        assert_eq!(findings.len(), 1);
    }
}
