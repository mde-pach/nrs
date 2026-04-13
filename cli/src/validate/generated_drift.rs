use super::{Finding, Validator, ValidatorScope};
use crate::generators::{self, Generator};
use crate::model::ContextSet;

/// Validator that checks generated files match what the generator would produce.
pub struct GeneratedDriftValidator {
    generator: Box<dyn Generator>,
}

impl GeneratedDriftValidator {
    pub fn new(generator: Box<dyn Generator>) -> Self {
        Self { generator }
    }
}

impl Validator for GeneratedDriftValidator {
    fn name(&self) -> &str {
        "generated_drift"
    }

    fn scope(&self) -> ValidatorScope {
        ValidatorScope::Global
    }

    fn check_all(&self, ctx_set: &ContextSet) -> anyhow::Result<Vec<Finding>> {
        check(ctx_set, self.generator.as_ref())
    }
}

/// Check that generated files match what the generator would produce.
fn check(ctx_set: &ContextSet, gen: &dyn Generator) -> anyhow::Result<Vec<Finding>> {
    let mut findings = Vec::new();

    for dir_ctx in &ctx_set.directories {
        let output_path = dir_ctx.dir.join(gen.output_filename());
        let relative = output_path
            .strip_prefix(&ctx_set.root)
            .unwrap_or(&output_path)
            .to_string_lossy()
            .to_string();

        let expected = generators::generate_and_rewrite(gen, dir_ctx, ctx_set);

        match std::fs::read_to_string(&output_path) {
            Ok(actual) => {
                if actual != expected {
                    findings.push(Finding::error(
                        &relative,
                        "generated file is out of date — run `nrs generate` to update",
                    ));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                findings.push(Finding::error(
                    &relative,
                    "generated file is missing — run `nrs generate` to create",
                ));
            }
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "failed to read {}: {}",
                    output_path.display(),
                    e
                ));
            }
        }
    }

    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generators::claude::ClaudeGenerator;
    use crate::model::{ContextFile, DirectoryContext, Layer};
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_ctx_set(dir: PathBuf, files: Vec<(&str, &str)>) -> ContextSet {
        let mut ctx_files: Vec<ContextFile> = files
            .into_iter()
            .map(|(name, content)| ContextFile {
                relative_path: name.to_string(),
                filename: name.to_string(),
                layer: Layer::from_filename(name),
                content: content.to_string(),
            })
            .collect();
        ctx_files.sort_by_key(|f| f.layer.sort_priority());

        let root = dir.clone();
        ContextSet {
            root,
            directories: vec![DirectoryContext {
                dir,
                relative_dir: String::new(),
                files: ctx_files,
            }],
        }
    }

    #[test]
    fn matching_generated_file_passes() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        let ctx_set = make_ctx_set(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "# Project")],
        );

        let expected = generators::generate_and_rewrite(&gen, &ctx_set.directories[0], &ctx_set);
        std::fs::write(tmp.path().join("CLAUDE.md"), &expected).unwrap();

        let findings = check(&ctx_set, &gen).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn mismatched_generated_file_is_error() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        let ctx_set = make_ctx_set(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "# Project")],
        );

        std::fs::write(tmp.path().join("CLAUDE.md"), "hand edited").unwrap();

        let findings = check(&ctx_set, &gen).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("out of date"));
    }

    #[test]
    fn missing_generated_file_is_error() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        let ctx_set = make_ctx_set(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "# Project")],
        );

        let findings = check(&ctx_set, &gen).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("missing"));
    }
}
