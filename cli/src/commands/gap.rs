use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

const VALID_TYPES: &[&str] = &["missing-context", "missing-concept", "missing-pattern", "wrong"];

const GAPS_FILENAME: &str = "nrs.gaps.md";

pub fn run_report(dir: &Path, gap_type: &str, target: &str, description: &str) -> Result<()> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("directory not found: {}", dir.display()))?;

    if !VALID_TYPES.contains(&gap_type) {
        anyhow::bail!(
            "invalid gap type '{}'. valid types: {}",
            gap_type,
            VALID_TYPES.join(", ")
        );
    }

    if target.trim().is_empty() {
        anyhow::bail!("--target must not be empty");
    }

    if description.trim().is_empty() {
        anyhow::bail!("--description must not be empty");
    }

    let gaps_path = dir.join(GAPS_FILENAME);
    let needs_header = !gaps_path.exists();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gaps_path)
        .with_context(|| format!("failed to open {}", gaps_path.display()))?;

    if needs_header {
        writeln!(file, "# NRS Gaps")?;
        writeln!(file)?;
        writeln!(file, "| Type | Target | Description |")?;
        writeln!(file, "|---|---|---|")?;
    }

    writeln!(file, "| {} | {} | {} |", gap_type, target, description)?;

    println!("reported {} gap for {}", gap_type, target);
    Ok(())
}

pub fn run_summary(dir: &Path) -> Result<()> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("directory not found: {}", dir.display()))?;

    let gaps_path = dir.join(GAPS_FILENAME);
    if !gaps_path.exists() {
        println!("no gaps reported");
        return Ok(());
    }

    let content = std::fs::read_to_string(&gaps_path)
        .with_context(|| format!("failed to read {}", gaps_path.display()))?;

    let mut gaps: Vec<(String, String, String)> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with('|') || line.starts_with("| Type") || line.starts_with("|---") {
            continue;
        }
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if parts.len() == 3 {
            gaps.push((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ));
        }
    }

    if gaps.is_empty() {
        println!("no gaps reported");
        return Ok(());
    }

    let mut by_target: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::new();
    for (gap_type, target, desc) in &gaps {
        by_target
            .entry(target.as_str())
            .or_default()
            .push((gap_type.as_str(), desc.as_str()));
    }

    for (target, entries) in &by_target {
        println!(
            "{} ({} gap{})",
            target,
            entries.len(),
            if entries.len() == 1 { "" } else { "s" }
        );
        for (gap_type, desc) in entries {
            println!("  [{}] {}", gap_type, desc);
        }
        println!();
    }

    println!("{} total gap(s)", gaps.len());
    Ok(())
}
