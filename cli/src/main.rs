mod commands;
mod discovery;
mod gaps;
mod generators;
mod markdown;
mod model;
mod observe;
mod validate;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nrs", about = "NRS context framework CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate tool-specific context files from NRS layers
    Generate {
        /// Target generator (e.g., claude, all)
        target: String,

        /// Project root directory
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },
    /// Validate NRS context files
    Validate {
        /// Project root directory
        #[arg(long, default_value = ".")]
        dir: PathBuf,

        /// Treat warnings as errors (exit code 1)
        #[arg(long)]
        strict: bool,
    },
    /// Initialize NRS precommit hooks in the project
    Init {
        /// Project root directory
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },
    /// Install NRS skill/config for agentic tools
    Install {
        /// Target tool (e.g., claude, all)
        target: String,
    },
    /// Claude Code integration
    Claude {
        #[command(subcommand)]
        action: ClaudeAction,
    },
    /// Report and view context gaps
    Gap {
        #[command(subcommand)]
        action: GapAction,
    },
}

#[derive(Subcommand)]
enum ClaudeAction {
    /// Observe agent behavior from a transcript and report signals as gaps
    Observe {
        /// Path to a Claude Code transcript JSONL file
        #[arg(long)]
        transcript: Option<std::path::PathBuf>,

        /// Project root directory
        #[arg(long, default_value = ".")]
        dir: PathBuf,

        /// Print detected signals without writing to nrs.gaps.md
        #[arg(long)]
        dry_run: bool,

        /// Read hook JSON from stdin (used by Claude Code SubagentStop/SessionEnd hooks)
        #[arg(long)]
        hook_mode: bool,
    },
    /// Block edits to generated files and suggest gap reporting
    Guard {
        /// Read hook JSON from stdin (used by Claude Code PreToolUse hook)
        #[arg(long)]
        hook_mode: bool,
    },
    /// List CLAUDE.md files with their NRS layer descriptions
    Layers {
        /// Project root directory
        #[arg(long, default_value = ".")]
        dir: PathBuf,

        /// Read hook JSON from stdin (used by Claude Code hooks)
        #[arg(long)]
        hook_mode: bool,
    },
    /// Notify agent about observed context gaps after task completion
    Notify {
        /// Project root directory
        #[arg(long, default_value = ".")]
        dir: PathBuf,

        /// Read hook JSON from stdin (used by Claude Code TaskCompleted hook)
        #[arg(long)]
        hook_mode: bool,
    },
}

#[derive(Subcommand)]
enum GapAction {
    /// Report a context gap
    Report {
        /// Gap type: missing-context, missing-concept, missing-pattern, wrong
        #[arg(long, value_name = "TYPE")]
        r#type: String,

        /// Target path (file or directory the gap relates to)
        #[arg(long)]
        target: String,

        /// Description of the gap
        #[arg(long)]
        description: String,

        /// Project root directory
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },
    /// Display gap summary grouped by target
    Summary {
        /// Project root directory
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate { target, dir } => commands::generate::run(&target, &dir),
        Command::Validate { dir, strict } => commands::validate::run(&dir, strict),
        Command::Init { dir } => commands::init::run(&dir),
        Command::Install { target } => commands::install::run(&target),
        Command::Claude { action } => match action {
            ClaudeAction::Observe {
                transcript,
                dir,
                dry_run,
                hook_mode,
            } => commands::observe::run(transcript.as_deref(), &dir, dry_run, hook_mode),
            ClaudeAction::Guard { hook_mode } => {
                commands::guard::run(hook_mode)
            }
            ClaudeAction::Layers { dir, hook_mode } => {
                commands::layers::run(&dir, hook_mode)
            }
            ClaudeAction::Notify { dir, hook_mode } => {
                commands::notify::run(&dir, hook_mode)
            }
        },
        Command::Gap { action } => match action {
            GapAction::Report {
                r#type,
                target,
                description,
                dir,
            } => commands::gap::run_report(&dir, &r#type, &target, &description),
            GapAction::Summary { dir } => commands::gap::run_summary(&dir),
        },
    }
}
