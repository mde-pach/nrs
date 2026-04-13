pub mod duplication;
pub mod generated_drift;
pub mod links;
pub mod orphan_docs;
pub mod references;
pub mod required_sections;
pub mod size;
pub mod source_paths;

use crate::generators::Generator;
use crate::model::{ContextFile, ContextSet, DirectoryContext};
use std::path::Path;

/// A single validation finding.
pub struct Finding {
    pub file: String,
    pub kind: FindingKind,
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub enum FindingKind {
    Warning,
    Error,
}

impl Finding {
    pub fn warning(file: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            kind: FindingKind::Warning,
            message: message.into(),
        }
    }

    pub fn error(file: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            kind: FindingKind::Error,
            message: message.into(),
        }
    }

    pub fn display(&self) -> String {
        let prefix = match self.kind {
            FindingKind::Warning => "warning",
            FindingKind::Error => "error",
        };
        format!("{}: {} — {}", prefix, self.file, self.message)
    }
}

/// The scope at which a validator operates.
pub enum ValidatorScope {
    /// Called once per context file.
    PerFile,
    /// Called once per directory containing context files.
    PerDirectory,
    /// Called once with the entire context set.
    Global,
}

/// Trait for pluggable validation checks.
///
/// Each validator declares its scope and implements the corresponding method.
/// Adding a new validator requires: create the struct, implement this trait,
/// add it to `all_validators()`.
pub trait Validator {
    #[allow(dead_code)]
    fn name(&self) -> &str;
    fn scope(&self) -> ValidatorScope;

    fn check_file(&self, _file: &ContextFile) -> Vec<Finding> {
        vec![]
    }

    fn check_directory(&self, _root: &Path, _dir: &DirectoryContext) -> Vec<Finding> {
        vec![]
    }

    fn check_all(&self, _ctx: &ContextSet) -> anyhow::Result<Vec<Finding>> {
        Ok(vec![])
    }
}

/// Build the list of all validators.
///
/// Generators are passed in so that `generated_drift` can compare output
/// without the validation module importing the generator registry.
pub fn all_validators(generators: Vec<Box<dyn Generator>>) -> Vec<Box<dyn Validator>> {
    let mut validators: Vec<Box<dyn Validator>> = vec![
        Box::new(size::SizeValidator),
        Box::new(source_paths::SourcePathsValidator),
        Box::new(references::ReferencesValidator),
        Box::new(links::LinksValidator),
        Box::new(duplication::DuplicationValidator),
        Box::new(orphan_docs::OrphanDocsValidator),
        Box::new(required_sections::RequiredSectionsValidator),
    ];

    for gen in generators {
        validators.push(Box::new(generated_drift::GeneratedDriftValidator::new(gen)));
    }

    validators
}

/// Run all validation checks and return findings.
pub fn run_all(
    ctx_set: &ContextSet,
    validators: &[Box<dyn Validator>],
) -> anyhow::Result<Vec<Finding>> {
    let mut findings = Vec::new();

    for validator in validators {
        match validator.scope() {
            ValidatorScope::PerFile => {
                for file in ctx_set.all_files() {
                    findings.extend(validator.check_file(file));
                }
            }
            ValidatorScope::PerDirectory => {
                for dir_ctx in &ctx_set.directories {
                    findings.extend(validator.check_directory(&ctx_set.root, dir_ctx));
                }
            }
            ValidatorScope::Global => {
                findings.extend(validator.check_all(ctx_set)?);
            }
        }
    }

    Ok(findings)
}
