//! Integration tests for `nrs generate`.

mod common;

use assert_cmd::Command;
use common::write_file;
use predicates::str::contains;
use tempfile::TempDir;

#[test]
fn generate_claude_writes_claude_md_at_root() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "project.context.md",
        "# Project\n\nA test project.\n",
    );

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("CLAUDE.md"));

    let claude = tmp.path().join("CLAUDE.md");
    assert!(claude.exists());
    let body = std::fs::read_to_string(&claude).unwrap();
    assert!(body.starts_with("<!-- DO NOT EDIT"));
    assert!(body.contains("# Project"));
}

#[test]
fn generate_claude_writes_claude_md_in_each_context_dir() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "project.context.md", "# Project\n");
    write_file(
        tmp.path(),
        "src/billing/domain.context.md",
        "# Billing domain\n",
    );

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    assert!(tmp.path().join("CLAUDE.md").exists());
    assert!(tmp.path().join("src/billing/CLAUDE.md").exists());

    let billing = std::fs::read_to_string(tmp.path().join("src/billing/CLAUDE.md")).unwrap();
    assert!(billing.contains("# Billing domain"));
}

#[test]
fn generate_orders_layers_correctly() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "nrs.context.md", "# NRS rules\n");
    write_file(tmp.path(), "corporate.context.md", "# Corporate\n");
    write_file(tmp.path(), "team.context.md", "# Team\n");
    write_file(tmp.path(), "project.context.md", "# Project\n");

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    let body = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    let nrs = body.find("# NRS rules").unwrap();
    let corp = body.find("# Corporate").unwrap();
    let team = body.find("# Team").unwrap();
    let proj = body.find("# Project").unwrap();
    assert!(
        nrs < corp && corp < team && team < proj,
        "wrong order: {body}"
    );
}

#[test]
fn generate_writes_claude_settings_with_ignore_pattern() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "project.context.md", "# Project\n");

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    let settings_path = tmp.path().join(".claude/settings.local.json");
    assert!(settings_path.exists());
    let body = std::fs::read_to_string(&settings_path).unwrap();
    assert!(body.contains("ignorePatterns"));
    assert!(body.contains("*.context.md"));
}

#[test]
fn generate_all_runs_every_generator() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "project.context.md", "# Project\n");

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "all", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    // Today: only Claude. When more generators are added, this test will catch
    // the missing output and remind us to assert on the new files.
    assert!(tmp.path().join("CLAUDE.md").exists());
}

#[test]
fn generate_unknown_target_errors() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "project.context.md", "# Project\n");

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "cursor", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stderr(contains("unknown generator"))
        .stderr(contains("claude"));
}

// ── Context link rewriting ─────────────────────────────────────────

#[test]
fn generate_rewrites_context_links_in_claude_md() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "project.context.md",
        "# Project\n\n- [Orders](src/orders/domain.context.md)\n",
    );
    write_file(
        tmp.path(),
        "src/orders/domain.context.md",
        "# Domain Context — Orders\n\nOrder business rules.\n",
    );

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    let body = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(
        body.contains("src/orders/CLAUDE.md#domain-context--orders"),
        "context link should be rewritten in CLAUDE.md, got: {body}"
    );
    assert!(
        !body.contains("domain.context.md"),
        "original context link should be gone from CLAUDE.md"
    );
}

#[test]
fn generate_rewrites_same_dir_context_link_to_anchor() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "src/billing/domain.context.md",
        "# Domain Context — Billing\n\nSee [Impl](implementation.context.md).\n",
    );
    write_file(
        tmp.path(),
        "src/billing/implementation.context.md",
        "# Implementation Context\n\nPatterns.\n",
    );
    write_file(tmp.path(), "project.context.md", "# Project\n");

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    let body =
        std::fs::read_to_string(tmp.path().join("src/billing/CLAUDE.md")).unwrap();
    assert!(
        body.contains("CLAUDE.md#implementation-context"),
        "same-dir context link should become anchor, got: {body}"
    );
}

#[test]
fn generate_preserves_docs_links() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "project.context.md",
        "# Project\n\n- [Testing](docs/testing.md)\n",
    );

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    let body = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(
        body.contains("[Testing](docs/testing.md)"),
        "docs link should be preserved"
    );
}

#[test]
fn generate_claude_installs_all_hooks() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "project.context.md", "# Project\n");

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    let settings_path = tmp.path().join(".claude/settings.json");
    assert!(settings_path.exists(), "settings.json should be created");
    let body = std::fs::read_to_string(&settings_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();

    // SubagentStop → observe
    let subagent = parsed["hooks"]["SubagentStop"].as_array().unwrap();
    assert_eq!(subagent.len(), 1);
    assert_eq!(
        subagent[0]["hooks"][0]["command"].as_str().unwrap(),
        "nrs claude observe --hook-mode"
    );

    // TaskCompleted → notify
    let task = parsed["hooks"]["TaskCompleted"].as_array().unwrap();
    assert_eq!(task.len(), 1);
    assert_eq!(
        task[0]["hooks"][0]["command"].as_str().unwrap(),
        "nrs claude notify --hook-mode"
    );

    // PreToolUse → guard
    let pre = parsed["hooks"]["PreToolUse"].as_array().unwrap();
    assert_eq!(pre.len(), 1);
    assert_eq!(
        pre[0]["hooks"][0]["command"].as_str().unwrap(),
        "nrs claude guard --hook-mode"
    );
    assert_eq!(pre[0]["matcher"].as_str().unwrap(), "Edit|Write");

    // FileChanged → generate + validate
    let file_changed = parsed["hooks"]["FileChanged"].as_array().unwrap();
    assert_eq!(file_changed.len(), 1);
    assert_eq!(
        file_changed[0]["hooks"][0]["command"].as_str().unwrap(),
        "nrs generate claude && nrs validate"
    );
    assert_eq!(file_changed[0]["matcher"].as_str().unwrap(), "*.context.md");

    // SessionStart → gap summary + validate
    let session_start = parsed["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 1);
    assert_eq!(
        session_start[0]["hooks"][0]["command"].as_str().unwrap(),
        "nrs gap summary && nrs validate"
    );

    // SessionEnd → observe
    let session_end = parsed["hooks"]["SessionEnd"].as_array().unwrap();
    assert_eq!(session_end.len(), 1);
    assert_eq!(
        session_end[0]["hooks"][0]["command"].as_str().unwrap(),
        "nrs claude observe --hook-mode"
    );

    // PreCompact → layers
    let pre_compact = parsed["hooks"]["PreCompact"].as_array().unwrap();
    assert_eq!(pre_compact.len(), 1);
    assert_eq!(
        pre_compact[0]["hooks"][0]["command"].as_str().unwrap(),
        "nrs claude layers --hook-mode"
    );

    // PostCompact → layers
    let post_compact = parsed["hooks"]["PostCompact"].as_array().unwrap();
    assert_eq!(post_compact.len(), 1);
    assert_eq!(
        post_compact[0]["hooks"][0]["command"].as_str().unwrap(),
        "nrs claude layers --hook-mode"
    );

    // SubagentStart → layers
    let subagent_start = parsed["hooks"]["SubagentStart"].as_array().unwrap();
    assert_eq!(subagent_start.len(), 1);
    assert_eq!(
        subagent_start[0]["hooks"][0]["command"].as_str().unwrap(),
        "nrs claude layers --hook-mode"
    );
}

#[test]
fn generate_with_no_context_files_is_a_no_op() {
    let tmp = TempDir::new().unwrap();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("no context files found"));

    assert!(!tmp.path().join("CLAUDE.md").exists());
}
