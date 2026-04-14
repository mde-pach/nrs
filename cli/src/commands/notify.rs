use anyhow::{Context, Result};
use std::path::Path;

use crate::gaps::{self, GAPS_FILENAME};

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
        anyhow::ensure!(dir.exists(), "directory not found: {}", dir.display());
        dir.to_path_buf()
    };

    let gaps_path = project_root.join(GAPS_FILENAME);
    if !gaps_path.exists() {
        return Ok(());
    }

    let all_gaps = gaps::parse_gaps_file(&gaps_path)?;
    let observed: Vec<&gaps::Gap> = all_gaps
        .iter()
        .filter(|g| g.source.starts_with("observed:"))
        .collect();

    if observed.is_empty() {
        return Ok(());
    }

    let mut summary = String::from(
        "NRS detected context gaps from agent behavior signals. Consider running the nrs-fix skill to address them.\n\nObserved gaps:\n",
    );
    for gap in &observed {
        summary.push_str(&format!(
            "- [{}] {} — {} ({})\n",
            gap.gap_type, gap.target, gap.description, gap.source
        ));
    }

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
