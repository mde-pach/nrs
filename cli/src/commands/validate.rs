use anyhow::{Context, Result};
use std::path::Path;

use crate::discovery;
use crate::generators;
use crate::validate::{self, FindingKind};

pub fn run(dir: &Path, strict: bool) -> Result<()> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("directory not found: {}", dir.display()))?;

    let ctx_set = discovery::discover(&dir)?;
    let validators = validate::all_validators(generators::all_generators());
    let findings = validate::run_all(&ctx_set, &validators)?;

    if findings.is_empty() {
        println!("ok");
        return Ok(());
    }

    let mut has_errors = false;
    let mut has_warnings = false;
    for f in &findings {
        println!("{}", f.display());
        match f.kind {
            FindingKind::Error => has_errors = true,
            FindingKind::Warning => has_warnings = true,
        }
    }

    let warnings = findings.iter().filter(|f| f.kind == FindingKind::Warning).count();
    let errors = findings.iter().filter(|f| f.kind == FindingKind::Error).count();

    println!();
    if errors > 0 {
        println!("{} error(s), {} warning(s)", errors, warnings);
    } else {
        println!("{} warning(s)", warnings);
    }

    let should_fail = has_errors || (strict && has_warnings);
    if should_fail {
        std::process::exit(1);
    }

    Ok(())
}
