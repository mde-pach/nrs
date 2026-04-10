pub mod claude;

use crate::context::DirectoryContext;
use std::path::Path;

pub trait Generator {
    fn name(&self) -> &str;
    fn output_filename(&self) -> &str;
    fn generate(&self, ctx: &DirectoryContext) -> String;

    /// Glob patterns the target tool should ignore (to avoid reading both
    /// the generated file and the source context files).
    fn ignore_patterns(&self) -> Vec<&str> {
        vec![]
    }

    /// Apply ignore patterns to the tool's configuration at the project root.
    /// Each generator knows its tool's config format.
    fn apply_ignores(&self, _project_root: &Path) -> anyhow::Result<()> {
        Ok(())
    }
}

pub fn all_generators() -> Vec<Box<dyn Generator>> {
    vec![Box::new(claude::ClaudeGenerator)]
}
