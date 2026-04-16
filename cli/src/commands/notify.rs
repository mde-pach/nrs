use anyhow::{Context, Result};
use std::path::Path;

use crate::gaps::{self, CANDIDATES_FILENAME};

pub fn run(dir: &Path, hook_mode: bool) -> Result<()> {
    let nrs_bin = std::env::current_exe()
        .unwrap_or_else(|_| std::path::PathBuf::from("nrs"))
        .display()
        .to_string();
    let mut hook_event_name = String::from("UserPromptSubmit");
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
        anyhow::ensure!(dir.exists(), "directory not found: {}", dir.display());
        dir.to_path_buf()
    };

    let candidates_path = project_root.join(CANDIDATES_FILENAME);
    if !candidates_path.exists() {
        return Ok(());
    }

    let mut all_gaps = gaps::parse_gaps_file(&candidates_path)?;
    if all_gaps.is_empty() {
        return Ok(());
    }

    // Pop the first candidate.
    let gap = all_gaps.remove(0);
    let remaining = all_gaps.len();

    let mut summary = format!(
        "NRS detected a context gap from agent behavior signals ({}, confidence: {}).\n\n\
         [{}] {} — {}\n\n\
         If this gap is relevant, run the command below in a background sub-agent. Otherwise, ignore it.\n\n\
         ```\n{} gap report --type {} --target \"{}\" --description \"{}\"\n```",
        gap.source,
        gap.confidence,
        gap.gap_type,
        gap.target,
        gap.description,
        nrs_bin,
        gap.gap_type,
        gap.target,
        gap.description.replace('"', "\\\""),
    );

    if remaining > 0 {
        summary.push_str(&format!(
            "\n\n({} more candidate(s) pending — will surface on next prompt.)",
            remaining
        ));
    }

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

    // Write remaining candidates back, or remove the file if empty.
    if all_gaps.is_empty() {
        std::fs::remove_file(&candidates_path).ok();
    } else {
        gaps::write_gaps_file(&candidates_path, &all_gaps)?;
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
