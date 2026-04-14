//! Integration tests for `nrs claude notify`.

use assert_cmd::Command;
use predicates::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

fn nrs() -> Command {
    Command::cargo_bin("nrs").unwrap()
}

#[test]
fn notify_silent_when_no_gaps_file() {
    let tmp = TempDir::new().unwrap();

    let output = nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}

#[test]
fn notify_silent_when_only_manual_gaps() {
    let tmp = TempDir::new().unwrap();
    let content = "# NRS Gaps\n\n\
| Type | Target | Description | Source | Confidence |\n\
|------|--------|-------------|--------|------------|\n\
| missing-context | src/billing/ | manual entry | manual | - |\n";
    std::fs::write(tmp.path().join("nrs.gaps.md"), content).unwrap();

    let output = nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    // Notify only surfaces observed gaps; manual gaps are silent.
    assert!(output.stdout.is_empty());
}

#[test]
fn notify_surfaces_observed_gaps() {
    let tmp = TempDir::new().unwrap();
    let content = "# NRS Gaps\n\n\
| Type | Target | Description | Source | Confidence |\n\
|------|--------|-------------|--------|------------|\n\
| missing-context | src/billing/ | manual entry | manual | - |\n\
| missing-pattern | src/billing/ | agent re-read | observed:re-reads | medium |\n";
    std::fs::write(tmp.path().join("nrs.gaps.md"), content).unwrap();

    nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("observed:re-reads"))
        .stdout(contains("nrs-fix"))
        // Manual gaps should not appear
        .stdout(predicates::str::contains("manual entry").not());
}

#[test]
fn notify_hook_mode_outputs_additional_context_json() {
    let tmp = TempDir::new().unwrap();
    let content = "# NRS Gaps\n\n\
| Type | Target | Description | Source | Confidence |\n\
|------|--------|-------------|--------|------------|\n\
| missing-pattern | src/billing/ | agent re-read | observed:re-reads | medium |\n";
    std::fs::write(tmp.path().join("nrs.gaps.md"), content).unwrap();

    let input = serde_json::json!({ "cwd": tmp.path().to_str().unwrap() });

    let output = nrs()
        .args(["claude", "notify", "--hook-mode"])
        .write_stdin(serde_json::to_string(&input).unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let ctx = parsed["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(ctx.contains("observed:re-reads"));
    assert!(ctx.contains("nrs-fix"));
}

#[test]
fn notify_hook_mode_silent_when_no_observed_gaps() {
    let tmp = TempDir::new().unwrap();
    let input = serde_json::json!({ "cwd": tmp.path().to_str().unwrap() });

    let output = nrs()
        .args(["claude", "notify", "--hook-mode"])
        .write_stdin(serde_json::to_string(&input).unwrap())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}
