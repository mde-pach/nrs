use super::Layer;

/// A single NRS context file with pre-computed metadata.
#[allow(dead_code)]
pub struct ContextFile {
    /// Path relative to project root (e.g. "src/billing/domain.context.md").
    pub relative_path: String,
    /// Just the filename (e.g. "domain.context.md").
    pub filename: String,
    /// The NRS layer this file belongs to.
    pub layer: Layer,
    /// File content.
    pub content: String,
}
