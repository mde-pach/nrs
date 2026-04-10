use super::Finding;
use crate::context::DirectoryContext;
use crate::generators::Generator;
use std::path::Path;

/// Check that generated files match what the generator would produce.
/// Compares actual file on disk with freshly generated content.
pub fn check(
    root: &Path,
    contexts: &[DirectoryContext],
    gen: &dyn Generator,
) -> anyhow::Result<Vec<Finding>> {
    let mut findings = Vec::new();

    for ctx in contexts {
        let output_path = ctx.dir.join(gen.output_filename());
        let relative = output_path
            .strip_prefix(root)
            .unwrap_or(&output_path)
            .to_string_lossy()
            .to_string();

        let expected = gen.generate(ctx);

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
    use crate::context::DirectoryContext;
    use crate::generators::claude::ClaudeGenerator;
    use crate::generators::Generator;
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn make_ctx(dir: PathBuf, files: Vec<(&str, &str)>) -> DirectoryContext {
        let mut map = BTreeMap::new();
        for (name, content) in files {
            map.insert(name.to_string(), content.to_string());
        }
        DirectoryContext { dir, files: map }
    }

    #[test]
    fn matching_generated_file_passes() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        let ctx = make_ctx(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "# Project")],
        );

        let expected = gen.generate(&ctx);
        std::fs::write(tmp.path().join("CLAUDE.md"), &expected).unwrap();

        let findings = check(tmp.path(), &[ctx], &gen).unwrap();
        assert!(findings.is_empty());
    }

    #[test]
    fn mismatched_generated_file_is_error() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        let ctx = make_ctx(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "# Project")],
        );

        std::fs::write(tmp.path().join("CLAUDE.md"), "hand edited").unwrap();

        let findings = check(tmp.path(), &[ctx], &gen).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("out of date"));
    }

    #[test]
    fn missing_generated_file_is_error() {
        let tmp = TempDir::new().unwrap();
        let gen = ClaudeGenerator;
        let ctx = make_ctx(
            tmp.path().to_path_buf(),
            vec![("project.context.md", "# Project")],
        );

        let findings = check(tmp.path(), &[ctx], &gen).unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].message.contains("missing"));
    }
}
