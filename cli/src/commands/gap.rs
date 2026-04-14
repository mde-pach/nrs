use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

use crate::gaps::{self, Gap, GAPS_FILENAME, VALID_TYPES};

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
    let gap = Gap::manual(gap_type, target, description);
    gaps::append_gap(&gaps_path, &gap)?;

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

    let all_gaps = gaps::parse_gaps_file(&gaps_path)?;

    if all_gaps.is_empty() {
        println!("no gaps reported");
        return Ok(());
    }

    let mut by_target: BTreeMap<&str, Vec<&Gap>> = BTreeMap::new();
    for gap in &all_gaps {
        by_target.entry(gap.target.as_str()).or_default().push(gap);
    }

    for (target, entries) in &by_target {
        println!(
            "{} ({} gap{})",
            target,
            entries.len(),
            if entries.len() == 1 { "" } else { "s" }
        );
        for gap in entries {
            if gap.source == "manual" {
                println!("  [{}] {}", gap.gap_type, gap.description);
            } else {
                println!("  [{}] ({}) {}", gap.gap_type, gap.source, gap.description);
            }
        }
        println!();
    }

    println!("{} total gap(s)", all_gaps.len());
    Ok(())
}
