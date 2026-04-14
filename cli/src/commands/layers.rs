use anyhow::{Context, Result};
use std::path::Path;

use crate::discovery;

pub fn run(dir: &Path, hook_mode: bool) -> Result<()> {
    let project_root = if hook_mode {
        let input = read_stdin()?;
        let value: serde_json::Value =
            serde_json::from_str(&input).context("failed to parse hook JSON from stdin")?;
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
    lines.push("NRS context layers in this project:".to_string());

    for dir_ctx in &ctx_set.directories {
        let layer_names: Vec<&str> = dir_ctx.files.iter().map(|f| f.layer.name()).collect();
        let claude_md = if dir_ctx.relative_dir.is_empty() {
            "CLAUDE.md".to_string()
        } else {
            format!("{}/CLAUDE.md", dir_ctx.relative_dir)
        };
        lines.push(format!("- {} — {}", claude_md, layer_names.join(", ")));
    }

    lines.push("Read the relevant CLAUDE.md for area-specific context.".to_string());

    let summary = lines.join("\n");

    if hook_mode {
        let output = serde_json::json!({
            "hookSpecificOutput": {
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
