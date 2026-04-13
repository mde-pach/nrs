use anyhow::{Context, Result};
use std::path::Path;

use crate::discovery;
use crate::generators;

pub fn run(target: &str, dir: &Path) -> Result<()> {
    let dir = dir
        .canonicalize()
        .with_context(|| format!("directory not found: {}", dir.display()))?;

    let all_generators = generators::all_generators();

    let targets: Vec<&dyn generators::Generator> = if target == "all" {
        all_generators.iter().map(|g| g.as_ref()).collect()
    } else {
        let gen = all_generators
            .iter()
            .find(|g| g.name() == target)
            .ok_or_else(|| {
                let available: Vec<&str> = all_generators.iter().map(|g| g.name()).collect();
                anyhow::anyhow!(
                    "unknown generator '{}'. available: {}",
                    target,
                    available.join(", ")
                )
            })?;
        vec![gen.as_ref()]
    };

    let ctx_set = discovery::discover(&dir)?;

    if ctx_set.directories.is_empty() {
        println!("no context files found");
        return Ok(());
    }

    for dir_ctx in &ctx_set.directories {
        for gen in &targets {
            let content = generators::generate_and_rewrite(*gen, dir_ctx, &ctx_set);
            let output_path = dir_ctx.dir.join(gen.output_filename());
            std::fs::write(&output_path, &content)
                .with_context(|| format!("failed to write {}", output_path.display()))?;

            let relative = output_path.strip_prefix(&dir).unwrap_or(&output_path);
            println!("wrote {}", relative.display());
        }
    }

    for gen in &targets {
        gen.apply_ignores(&dir)?;
    }

    Ok(())
}
