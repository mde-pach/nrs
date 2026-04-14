use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub const GAPS_FILENAME: &str = "nrs.gaps.md";

pub const VALID_TYPES: &[&str] = &["missing-context", "missing-concept", "missing-pattern", "wrong"];

const TABLE_HEADER: &str = "| Type | Target | Description | Source | Confidence |";
const TABLE_SEPARATOR: &str = "|---|---|---|---|---|";

pub struct Gap {
    pub gap_type: String,
    pub target: String,
    pub description: String,
    pub source: String,
    pub confidence: String,
}

impl Gap {
    pub fn manual(gap_type: &str, target: &str, description: &str) -> Self {
        Self {
            gap_type: gap_type.to_string(),
            target: target.to_string(),
            description: description.to_string(),
            source: "manual".to_string(),
            confidence: "-".to_string(),
        }
    }

    pub fn observed(
        gap_type: &str,
        target: &str,
        description: &str,
        pattern: &str,
        confidence: &str,
    ) -> Self {
        Self {
            gap_type: gap_type.to_string(),
            target: target.to_string(),
            description: description.to_string(),
            source: format!("observed:{}", pattern),
            confidence: confidence.to_string(),
        }
    }

    fn to_row(&self) -> String {
        format!(
            "| {} | {} | {} | {} | {} |",
            self.gap_type, self.target, self.description, self.source, self.confidence
        )
    }
}

/// Parse gaps from an nrs.gaps.md file.
pub fn parse_gaps_file(path: &Path) -> Result<Vec<Gap>> {
    let content =
        std::fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(parse_gaps(&content))
}

/// Parse gaps from markdown content.
pub fn parse_gaps(content: &str) -> Vec<Gap> {
    let mut gaps = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if !line.starts_with('|') || line.starts_with("| Type") || line.starts_with("|---") {
            continue;
        }
        let parts: Vec<&str> = line
            .split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() >= 5 {
            gaps.push(Gap {
                gap_type: parts[0].to_string(),
                target: parts[1].to_string(),
                description: parts[2].to_string(),
                source: parts[3].to_string(),
                confidence: parts[4].to_string(),
            });
        }
    }

    gaps
}

/// Append a single gap to the gaps file, creating it with a header if needed.
pub fn append_gap(path: &Path, gap: &Gap) -> Result<()> {
    let needs_header = !path.exists();

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;

    if needs_header {
        writeln!(file, "# NRS Gaps")?;
        writeln!(file)?;
        writeln!(file, "{}", TABLE_HEADER)?;
        writeln!(file, "{}", TABLE_SEPARATOR)?;
    }

    writeln!(file, "{}", gap.to_row())?;
    Ok(())
}

/// Rewrite the entire gaps file with the given gaps.
pub fn write_gaps_file(path: &Path, gaps: &[Gap]) -> Result<()> {
    let mut content = String::new();
    content.push_str("# NRS Gaps\n\n");
    content.push_str(TABLE_HEADER);
    content.push('\n');
    content.push_str(TABLE_SEPARATOR);
    content.push('\n');
    for gap in gaps {
        content.push_str(&gap.to_row());
        content.push('\n');
    }
    std::fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}
