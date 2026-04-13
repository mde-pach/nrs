use std::path::PathBuf;

use super::ContextFile;

/// Context files found in a single directory.
#[allow(dead_code)]
pub struct DirectoryContext {
    /// Absolute path to the directory.
    pub dir: PathBuf,
    /// Path relative to project root (empty string for root).
    pub relative_dir: String,
    /// Context files in this directory, ordered by layer priority.
    pub files: Vec<ContextFile>,
}

/// All discovered context files across the project.
pub struct ContextSet {
    /// Absolute path to the project root.
    pub root: PathBuf,
    /// Directories containing context files, sorted by path.
    pub directories: Vec<DirectoryContext>,
}

impl ContextSet {
    /// Iterate over every context file across all directories.
    pub fn all_files(&self) -> impl Iterator<Item = &ContextFile> {
        self.directories.iter().flat_map(|d| d.files.iter())
    }
}
