pub mod claude;

use crate::markdown;
use crate::model::{ContextSet, DirectoryContext};
use std::path::Path;

pub trait Generator {
    fn name(&self) -> &str;
    fn output_filename(&self) -> &str;
    fn generate(&self, ctx: &DirectoryContext) -> String;

    /// Apply permission rules (deny/allow) to the tool's configuration at the
    /// project root. Each generator knows its tool's config format.
    fn apply_permissions(&self, _project_root: &Path) -> anyhow::Result<()> {
        Ok(())
    }

    /// Install hooks for the tool at the project root.
    /// Each generator knows its tool's hook format.
    fn apply_hooks(&self, _project_root: &Path) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Generate content and rewrite `*.context.md` links to point to the
/// compiled output file (e.g. `CLAUDE.md#anchor`).
pub fn generate_and_rewrite(
    gen: &dyn Generator,
    dir_ctx: &DirectoryContext,
    ctx_set: &ContextSet,
) -> String {
    let raw = gen.generate(dir_ctx);
    markdown::rewrite_context_links(
        &raw,
        &dir_ctx.files,
        ctx_set,
        &dir_ctx.dir,
        gen.output_filename(),
    )
}

pub fn all_generators() -> Vec<Box<dyn Generator>> {
    vec![Box::new(claude::ClaudeGenerator)]
}
