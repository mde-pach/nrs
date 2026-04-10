pub mod size;
pub mod source_paths;
pub mod generated_drift;
pub mod links;
pub mod references;

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

/// Run all validation checks and return findings.
pub fn run_all(root: &Path) -> anyhow::Result<Vec<Finding>> {
    let contexts = crate::context::discover(root)?;
    let mut findings = Vec::new();

    for ctx in &contexts {
        let relative_dir = ctx
            .dir
            .strip_prefix(root)
            .unwrap_or(&ctx.dir)
            .to_string_lossy()
            .to_string();

        for (name, content) in &ctx.files {
            let relative = if relative_dir.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", relative_dir, name)
            };

            findings.extend(size::check(&relative, name, content));
            findings.extend(source_paths::check(&relative, content));
            findings.extend(references::check(&relative, name, content));
        }

        findings.extend(links::check(root, &ctx.dir, &relative_dir, &ctx.files));
    }

    let generators = crate::generators::all_generators();
    for gen in &generators {
        findings.extend(generated_drift::check(root, &contexts, gen.as_ref())?);
    }

    Ok(findings)
}
