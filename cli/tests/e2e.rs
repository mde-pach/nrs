//! End-to-end happy paths.

mod common;

use assert_cmd::Command;
use common::{git_repo, write_file};
use predicates::str::contains;
use std::io::Write;

#[test]
fn full_happy_path_init_generate_validate() {
    let tmp = git_repo();

    // 1. Initialize NRS in the project
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["init", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    // 2. Author a project map and a domain context
    write_file(
        tmp.path(),
        "project.context.md",
        "# Project Context — Demo\n\n## Purpose\n\nA test project.\n\n## Commands\n\n- `cargo test` — run tests\n",
    );
    write_file(
        tmp.path(),
        "src/users/domain.context.md",
        "# Users domain\n\nA user has an email and a role.\n",
    );

    // 3. Generate tool entry points
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "all", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    // CLAUDE.md should exist at root and inside the domain dir
    assert!(tmp.path().join("CLAUDE.md").exists());
    assert!(tmp.path().join("src/users/CLAUDE.md").exists());

    // The root CLAUDE.md should include the nrs.context.md template content
    let root_claude = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(root_claude.contains("Gap Reporting"));
    assert!(root_claude.contains("# Project Context"));

    // 4. Validate — should pass cleanly
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();
}

#[test]
fn validate_fails_after_authoring_violation_then_passes_after_fix() {
    let tmp = git_repo();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["init", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    // Author a domain context that leaks an implementation detail
    let domain_path = "src/billing/domain.context.md";
    write_file(
        tmp.path(),
        domain_path,
        "# Billing\n\nWe use Prisma to persist invoices.\n",
    );
    write_file(tmp.path(), "project.context.md", "# Project\n\n## Commands\n\n- `make build` — build\n");

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "all", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure();

    // Fix the violation
    std::fs::write(
        tmp.path().join(domain_path),
        "# Billing\n\nInvoices are persisted between checkout and fulfilment.\n",
    )
    .unwrap();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "all", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();
}

// ── Observe → Notify pipeline ────────────────────────────────────

fn write_transcript(dir: &std::path::Path, entries: &[serde_json::Value]) -> std::path::PathBuf {
    let path = dir.join("transcript.jsonl");
    let mut file = std::fs::File::create(&path).unwrap();
    for entry in entries {
        writeln!(file, "{}", serde_json::to_string(entry).unwrap()).unwrap();
    }
    path
}

fn tool_use(name: &str, file_path: &str) -> serde_json::Value {
    serde_json::json!({
        "role": "assistant",
        "content": [{
            "type": "tool_use",
            "name": name,
            "input": { "file_path": file_path }
        }]
    })
}

#[test]
fn observe_notify_pipeline_surfaces_and_clears_candidates() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    let src = project.join("src/billing");
    std::fs::create_dir_all(&src).unwrap();
    for i in 0..6 {
        std::fs::write(src.join(format!("file{}.rs", i)), "").unwrap();
    }

    // 1. Build a transcript that triggers excessive-reads in src/billing/
    let entries: Vec<serde_json::Value> = (0..6)
        .map(|i| {
            let abs = project.join(format!("src/billing/file{}.rs", i));
            tool_use("Read", abs.to_str().unwrap())
        })
        .collect();
    let transcript = write_transcript(tmp.path(), &entries);

    // 2. Run observe — should create candidates file
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .assert()
        .success()
        .stdout(contains("nrs.gaps.candidates.md"));

    let candidates_path = project.join("nrs.gaps.candidates.md");
    assert!(candidates_path.exists(), "candidates file should exist after observe");
    let candidates = std::fs::read_to_string(&candidates_path).unwrap();
    assert!(candidates.contains("observed:excessive-reads"));

    // nrs.gaps.md should NOT exist — observe no longer writes there
    assert!(
        !project.join("nrs.gaps.md").exists(),
        "observe should not write to nrs.gaps.md"
    );

    // 3. Run notify — should output summary and clear candidates
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["claude", "notify", "--dir"])
        .arg(&project)
        .assert()
        .success()
        .stdout(contains("context gap"))
        .stdout(contains("excessive-reads"))
        .stdout(contains("nrs gap report"));

    assert!(
        !candidates_path.exists(),
        "candidates file should be cleared after notify"
    );

    // 4. Run notify again — should be silent (no candidates left)
    let output = Command::cargo_bin("nrs")
        .unwrap()
        .args(["claude", "notify", "--dir"])
        .arg(&project)
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty(), "second notify should be silent");
}

#[test]
fn observe_notify_hook_mode_pipeline() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    let src = project.join("src/auth");
    std::fs::create_dir_all(&src).unwrap();

    // 3+ ops in a dir without context → no-context pattern
    let entries = vec![
        tool_use("Read", &format!("{}/src/auth/login.rs", project.display())),
        tool_use("Read", &format!("{}/src/auth/session.rs", project.display())),
        tool_use("Edit", &format!("{}/src/auth/login.rs", project.display())),
    ];
    let transcript = write_transcript(tmp.path(), &entries);

    // 1. Observe via hook-mode (simulates Stop hook)
    let observe_input = serde_json::json!({
        "transcript_path": transcript.to_str().unwrap(),
        "cwd": project.to_str().unwrap()
    });
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["claude", "observe", "--hook-mode"])
        .write_stdin(serde_json::to_string(&observe_input).unwrap())
        .assert()
        .success();

    assert!(project.join("nrs.gaps.candidates.md").exists());

    // 2. Notify via hook-mode (simulates UserPromptSubmit hook)
    let notify_input = serde_json::json!({
        "cwd": project.to_str().unwrap()
    });
    let output = Command::cargo_bin("nrs")
        .unwrap()
        .args(["claude", "notify", "--hook-mode"])
        .write_stdin(serde_json::to_string(&notify_input).unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let ctx = parsed["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(ctx.contains("no-context"), "should contain the detected pattern");
    assert!(ctx.contains("nrs gap report"), "should contain triage instruction");

    // Candidates should be cleared
    assert!(!project.join("nrs.gaps.candidates.md").exists());
}
