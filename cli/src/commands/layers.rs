use anyhow::{Context, Result};
use std::path::Path;

use crate::discovery;

pub fn run(dir: &Path, hook_mode: bool) -> Result<()> {
    let mut hook_event_name = String::new();
    let project_root = if hook_mode {
        let input = read_stdin()?;
        let value: serde_json::Value =
            serde_json::from_str(&input).context("failed to parse hook JSON from stdin")?;
        if let Some(name) = value.get("hook_event_name").and_then(|v| v.as_str()) {
            hook_event_name = name.to_string();
        }
        let cwd = value
            .get("cwd")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        std::path::PathBuf::from(cwd)
    } else {
        dir.canonicalize()
            .with_context(|| format!("directory not found: {}", dir.display()))?
    };

    let ctx_set = discovery::discover(&project_root)?;

    if ctx_set.directories.is_empty() {
        return Ok(());
    }

    let mut lines = Vec::new();
    lines.push("NRS context layers:".to_string());

    // Collect entries: (relative_dir, layer names).
    let entries: Vec<(&str, String)> = ctx_set
        .directories
        .iter()
        .map(|d| {
            let layer_names: Vec<&str> = d.files.iter().map(|f| f.layer.name()).collect();
            (d.relative_dir.as_str(), layer_names.join(", "))
        })
        .collect();

    // For each entry, find its nearest context-directory ancestor (tree parent)
    // and compute its tree depth. Only directories that actually have context
    // files appear as nodes — intermediate filesystem dirs are skipped.
    let all_dirs: Vec<&str> = entries.iter().map(|(d, _)| *d).collect();

    fn find_parent<'a>(dir: &'a str, all: &[&'a str]) -> Option<&'a str> {
        // Walk up the path until we find an ancestor that has context files.
        let mut path = dir;
        loop {
            match path.rfind('/') {
                Some(pos) => {
                    path = &path[..pos];
                    if all.contains(&path) {
                        return Some(path);
                    }
                }
                None => {
                    // Parent is root (empty string) if root exists.
                    if all.contains(&"") {
                        return Some("");
                    }
                    return None;
                }
            }
        }
    }

    fn tree_depth(dir: &str, all: &[&str]) -> usize {
        if dir.is_empty() {
            return 0;
        }
        let mut depth = 0;
        let mut current = dir;
        while let Some(parent) = find_parent(current, all) {
            depth += 1;
            if parent.is_empty() {
                break;
            }
            current = parent;
        }
        depth
    }

    // For each entry, find siblings (entries sharing the same parent).
    for (i, (rel_dir, layers)) in entries.iter().enumerate() {
        if rel_dir.is_empty() {
            lines.push(format!("CLAUDE.md ({})", layers));
            continue;
        }

        let depth = tree_depth(rel_dir, &all_dirs);
        let parent = find_parent(rel_dir, &all_dirs);

        // Label: path relative to the tree parent.
        let label = match parent {
            Some(p) if !p.is_empty() => rel_dir
                .strip_prefix(p)
                .and_then(|s| s.strip_prefix('/'))
                .unwrap_or(rel_dir),
            _ => *rel_dir,
        };

        // Is this the last sibling under its parent?
        let is_last = !entries[i + 1..].iter().any(|(other, _)| {
            find_parent(other, &all_dirs) == parent && other != rel_dir
        });

        // Build indent: for each ancestor level (1..depth), check if that
        // ancestor has remaining siblings after this subtree.
        let mut prefixes: Vec<&str> = Vec::new();
        let mut ancestor = *rel_dir;
        let mut ancestor_chain = Vec::new();
        for _ in 0..depth {
            ancestor_chain.push(ancestor);
            ancestor = find_parent(ancestor, &all_dirs).unwrap_or("");
        }
        ancestor_chain.reverse();

        for level in 0..depth - 1 {
            let anc = ancestor_chain[level];
            let anc_parent = find_parent(anc, &all_dirs);
            let anc_is_last = !entries.iter().any(|(other, _)| {
                *other != anc
                    && find_parent(other, &all_dirs) == anc_parent
                    && *other > anc
            });
            prefixes.push(if anc_is_last { "    " } else { "│   " });
        }

        let indent: String = prefixes.concat();
        let connector = if is_last { "└── " } else { "├── " };
        lines.push(format!("{}{}{}/CLAUDE.md ({})", indent, connector, label, layers));
    }

    lines.push("Read the relevant CLAUDE.md for area-specific context.".to_string());

    let summary = lines.join("\n");

    if hook_mode {
        let output = serde_json::json!({
            "hookSpecificOutput": {
                "hookEventName": hook_event_name,
                "additionalContext": summary
            }
        });
        println!("{}", serde_json::to_string(&output)?);
    } else {
        println!("{}", summary);
    }

    Ok(())
}

fn read_stdin() -> Result<String> {
    use std::io::Read;
    let mut buf = String::new();
    std::io::stdin()
        .read_to_string(&mut buf)
        .context("failed to read stdin")?;
    Ok(buf)
}
