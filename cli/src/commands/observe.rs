use anyhow::{Context, Result};
use std::path::Path;

use crate::gaps::{self, CANDIDATES_FILENAME};
use crate::observe;

pub fn run(transcript: Option<&Path>, dir: &Path, dry_run: bool, hook_mode: bool) -> Result<()> {
    let (transcript_path, project_root) = if hook_mode {
        let input = read_stdin()?;
        let value: serde_json::Value =
            serde_json::from_str(&input).context("failed to parse hook JSON from stdin")?;
        // SubagentStop provides agent_transcript_path; fall back to transcript_path
        let tp = value
            .get("agent_transcript_path")
            .or_else(|| value.get("transcript_path"))
            .and_then(|v| v.as_str())
            .context("missing agent_transcript_path or transcript_path in hook input")?
            .to_string();
        let cwd = value
            .get("cwd")
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        (tp, std::path::PathBuf::from(cwd))
    } else {
        let tp = transcript
            .context("--transcript is required when not in --hook-mode")?
            .to_string_lossy()
            .to_string();
        anyhow::ensure!(dir.exists(), "directory not found: {}", dir.display());
        (tp, dir.to_path_buf())
    };

    let content = std::fs::read_to_string(&transcript_path)
        .with_context(|| format!("failed to read transcript: {}", transcript_path))?;

    let events = observe::parse_transcript(&content, &project_root);
    let detected_gaps = observe::detect_patterns(&events, &project_root);

    if detected_gaps.is_empty() {
        if !hook_mode {
            println!("no signals detected");
        }
        return Ok(());
    }

    if dry_run {
        println!("{} signal(s) detected:\n", detected_gaps.len());
        for gap in &detected_gaps {
            println!(
                "  [{}] ({}) {} — {} (confidence: {})",
                gap.gap_type, gap.source, gap.target, gap.description, gap.confidence
            );
        }
        return Ok(());
    }

    let gaps_path = project_root.join(CANDIDATES_FILENAME);
    for gap in &detected_gaps {
        gaps::append_gap(&gaps_path, gap)?;
    }

    if !hook_mode {
        println!(
            "wrote {} signal(s) to {}",
            detected_gaps.len(),
            CANDIDATES_FILENAME
        );
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
