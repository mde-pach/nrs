use anyhow::{Context, Result};
use std::path::PathBuf;

const SETUP_SKILL_MD: &str = include_str!("../../../skill/nrs-setup/SKILL.md");
const CONTEXT_FORMATS: &str = include_str!("../../../skill/nrs-setup/references/context-formats.md");
const FIX_SKILL_MD: &str = include_str!("../../../skill/nrs-fix/SKILL.md");

struct Installer {
    name: &'static str,
    install: fn() -> Result<()>,
}

const INSTALLERS: &[Installer] = &[Installer {
    name: "claude",
    install: install_claude,
}];

pub fn run(target: &str) -> Result<()> {
    let targets: Vec<&Installer> = if target == "all" {
        INSTALLERS.iter().collect()
    } else {
        let installer = INSTALLERS
            .iter()
            .find(|i| i.name == target)
            .ok_or_else(|| {
                let available: Vec<&str> = INSTALLERS.iter().map(|i| i.name).collect();
                anyhow::anyhow!(
                    "unknown install target '{}'. available: {}",
                    target,
                    available.join(", ")
                )
            })?;
        vec![installer]
    };

    for installer in targets {
        (installer.install)()?;
    }

    Ok(())
}

fn install_claude() -> Result<()> {
    let home = std::env::var("HOME").context("HOME not set")?;

    let setup_dir = PathBuf::from(&home).join(".claude/skills/nrs-setup");
    let refs_dir = setup_dir.join("references");
    std::fs::create_dir_all(&refs_dir)
        .with_context(|| format!("failed to create {}", refs_dir.display()))?;
    std::fs::write(setup_dir.join("SKILL.md"), SETUP_SKILL_MD)?;
    std::fs::write(refs_dir.join("context-formats.md"), CONTEXT_FORMATS)?;
    println!("installed nrs-setup skill at {}", setup_dir.display());

    let fix_dir = PathBuf::from(&home).join(".claude/skills/nrs-fix");
    std::fs::create_dir_all(&fix_dir)
        .with_context(|| format!("failed to create {}", fix_dir.display()))?;
    std::fs::write(fix_dir.join("SKILL.md"), FIX_SKILL_MD)?;
    println!("installed nrs-fix skill at {}", fix_dir.display());

    Ok(())
}
