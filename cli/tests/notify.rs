//! Integration tests for `nrs claude notify`.

use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;

fn nrs() -> Command {
    Command::cargo_bin("nrs").unwrap()
}

fn two_gap_candidates() -> &'static str {
    "# NRS Gaps\n\n\
| Type | Target | Description | Source | Confidence |\n\
|---|---|---|---|---|\n\
| missing-pattern | src/billing/ | agent re-read | observed:re-reads | medium |\n\
| missing-context | src/auth/ | excessive reads | observed:excessive-reads | high |\n"
}

fn one_gap_candidate() -> &'static str {
    "# NRS Gaps\n\n\
| Type | Target | Description | Source | Confidence |\n\
|---|---|---|---|---|\n\
| missing-pattern | src/billing/ | agent re-read | observed:re-reads | medium |\n"
}

#[test]
fn notify_silent_when_no_candidates_file() {
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
fn notify_pops_first_candidate_only() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("nrs.gaps.candidates.md"), two_gap_candidates()).unwrap();

    nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("missing-pattern"))
        .stdout(contains("src/billing/"))
        .stdout(contains("nrs gap report --type missing-pattern --target"))
        .stdout(contains("1 more candidate(s) pending"));
}

#[test]
fn notify_does_not_surface_second_gap() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("nrs.gaps.candidates.md"), two_gap_candidates()).unwrap();

    let output = nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(!stdout.contains("src/auth/"), "second gap should not appear in first pop");
}

#[test]
fn notify_keeps_remaining_candidates_in_file() {
    let tmp = TempDir::new().unwrap();
    let candidates = tmp.path().join("nrs.gaps.candidates.md");
    std::fs::write(&candidates, two_gap_candidates()).unwrap();

    nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    assert!(candidates.exists(), "candidates file should still exist with remaining gaps");
    let remaining = std::fs::read_to_string(&candidates).unwrap();
    assert!(remaining.contains("src/auth/"), "second gap should remain in file");
    assert!(!remaining.contains("src/billing/"), "first gap should be removed");
}

#[test]
fn notify_removes_file_when_last_candidate_popped() {
    let tmp = TempDir::new().unwrap();
    let candidates = tmp.path().join("nrs.gaps.candidates.md");
    std::fs::write(&candidates, one_gap_candidate()).unwrap();

    nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("nrs gap report --type missing-pattern"));

    assert!(!candidates.exists(), "candidates file should be removed after last gap popped");
}

#[test]
fn notify_drains_all_candidates_over_multiple_calls() {
    let tmp = TempDir::new().unwrap();
    let candidates = tmp.path().join("nrs.gaps.candidates.md");
    std::fs::write(&candidates, two_gap_candidates()).unwrap();

    // First call — pops billing
    nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("src/billing/"));

    // Second call — pops auth
    nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("src/auth/"));

    // Third call — silent
    let output = nrs()
        .args(["claude", "notify", "--dir"])
        .arg(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(output.stdout.is_empty(), "third notify should be silent");
}

#[test]
fn notify_hook_mode_outputs_single_gap_json() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("nrs.gaps.candidates.md"), two_gap_candidates()).unwrap();

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
    assert!(ctx.contains("src/billing/"), "should contain the first gap");
    assert!(!ctx.contains("src/auth/"), "should not contain the second gap");
    assert!(ctx.contains("nrs gap report"));
    assert!(ctx.contains("1 more candidate(s) pending"));
}

#[test]
fn notify_hook_mode_silent_when_no_candidates() {
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
