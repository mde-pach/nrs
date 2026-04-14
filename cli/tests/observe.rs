//! Integration tests for `nrs observe`.

use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;

fn nrs() -> Command {
    Command::cargo_bin("nrs").unwrap()
}

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

fn user_msg(text: &str) -> serde_json::Value {
    serde_json::json!({
        "role": "user",
        "content": text
    })
}

#[test]
fn observe_dry_run_detects_excessive_reads() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    let src = project.join("src/billing");
    std::fs::create_dir_all(&src).unwrap();
    for i in 0..6 {
        std::fs::write(src.join(format!("file{}.rs", i)), "").unwrap();
    }

    let entries: Vec<serde_json::Value> = (0..6)
        .map(|i| {
            let abs = project.join(format!("src/billing/file{}.rs", i));
            tool_use("Read", abs.to_str().unwrap())
        })
        .collect();

    let transcript = write_transcript(tmp.path(), &entries);

    nrs()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(contains("signal(s) detected"))
        .stdout(contains("excessive-reads"));
}

#[test]
fn observe_writes_gaps_to_file() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    let src = project.join("src/billing");
    std::fs::create_dir_all(&src).unwrap();
    for i in 0..6 {
        std::fs::write(src.join(format!("file{}.rs", i)), "").unwrap();
    }

    let entries: Vec<serde_json::Value> = (0..6)
        .map(|i| {
            let abs = project.join(format!("src/billing/file{}.rs", i));
            tool_use("Read", abs.to_str().unwrap())
        })
        .collect();

    let transcript = write_transcript(tmp.path(), &entries);

    nrs()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .assert()
        .success()
        .stdout(contains("wrote"));

    let gaps = std::fs::read_to_string(project.join("nrs.gaps.md")).unwrap();
    assert!(gaps.contains("observed:excessive-reads"));
    assert!(gaps.contains("src/billing"));
}

#[test]
fn observe_no_signals_on_clean_transcript() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();

    // Agent reads 2 files and writes 1 — normal work, no signal
    let entries = vec![
        tool_use("Read", &format!("{}/src/foo.rs", project.display())),
        tool_use("Edit", &format!("{}/src/foo.rs", project.display())),
    ];

    let transcript = write_transcript(tmp.path(), &entries);

    nrs()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(contains("no signals detected"));
}

#[test]
fn observe_detects_no_context_pattern() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    let src = project.join("src/auth");
    std::fs::create_dir_all(&src).unwrap();

    let entries = vec![
        tool_use("Read", &format!("{}/src/auth/login.rs", project.display())),
        tool_use("Read", &format!("{}/src/auth/session.rs", project.display())),
        tool_use("Edit", &format!("{}/src/auth/login.rs", project.display())),
    ];

    let transcript = write_transcript(tmp.path(), &entries);

    nrs()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(contains("no-context"));
}

#[test]
fn observe_detects_user_correction() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();

    let entries = vec![
        user_msg("no, that's wrong"),
        tool_use("Edit", &format!("{}/src/billing/calc.rs", project.display())),
    ];

    let transcript = write_transcript(tmp.path(), &entries);

    nrs()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .arg("--dry-run")
        .assert()
        .success()
        .stdout(contains("user-correction"));
}

#[test]
fn observe_user_correction_writes_wrong_gap_to_file() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();

    let entries = vec![
        user_msg("no, that's wrong"),
        tool_use("Edit", &format!("{}/src/billing/calc.rs", project.display())),
    ];

    let transcript = write_transcript(tmp.path(), &entries);

    nrs()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .assert()
        .success();

    let gaps = std::fs::read_to_string(project.join("nrs.gaps.md")).unwrap();
    assert!(gaps.contains("observed:user-correction"));
    assert!(gaps.contains("wrong"));
    assert!(gaps.contains("src/billing"));
}

#[test]
fn observe_detects_re_reads_pattern() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    let src = project.join("src/billing");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("calc.rs"), "").unwrap();

    // Same file read 3+ times triggers re-reads pattern
    let abs = project.join("src/billing/calc.rs");
    let entries: Vec<serde_json::Value> = (0..3)
        .map(|_| tool_use("Read", abs.to_str().unwrap()))
        .collect();

    let transcript = write_transcript(tmp.path(), &entries);

    nrs()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .assert()
        .success();

    let gaps = std::fs::read_to_string(project.join("nrs.gaps.md")).unwrap();
    assert!(gaps.contains("observed:re-reads"));
    assert!(gaps.contains("missing-pattern"));
}

#[test]
fn observe_detects_backtracking_pattern() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    let src = project.join("src/orders");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(src.join("main.rs"), "").unwrap();
    std::fs::write(src.join("a.rs"), "").unwrap();
    std::fs::write(src.join("b.rs"), "").unwrap();

    let main = project.join("src/orders/main.rs");
    let a = project.join("src/orders/a.rs");
    let b = project.join("src/orders/b.rs");

    // write → read → read → rewrite same file
    let entries = vec![
        tool_use("Edit", main.to_str().unwrap()),
        tool_use("Read", a.to_str().unwrap()),
        tool_use("Read", b.to_str().unwrap()),
        tool_use("Edit", main.to_str().unwrap()),
    ];

    let transcript = write_transcript(tmp.path(), &entries);

    nrs()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .assert()
        .success();

    let gaps = std::fs::read_to_string(project.join("nrs.gaps.md")).unwrap();
    assert!(gaps.contains("observed:backtracking"));
}

#[test]
fn observe_detects_multiple_patterns_in_one_transcript() {
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    let billing = project.join("src/billing");
    let auth = project.join("src/auth");
    std::fs::create_dir_all(&billing).unwrap();
    std::fs::create_dir_all(&auth).unwrap();
    for i in 0..6 {
        std::fs::write(billing.join(format!("f{}.rs", i)), "").unwrap();
    }

    let mut entries: Vec<serde_json::Value> = (0..6)
        .map(|i| {
            let abs = project.join(format!("src/billing/f{}.rs", i));
            tool_use("Read", abs.to_str().unwrap())
        })
        .collect();
    // Plus 3 ops in src/auth/ with no context → no-context pattern
    entries.push(tool_use("Read", &format!("{}/src/auth/a.rs", project.display())));
    entries.push(tool_use("Read", &format!("{}/src/auth/b.rs", project.display())));
    entries.push(tool_use("Edit", &format!("{}/src/auth/a.rs", project.display())));

    let transcript = write_transcript(tmp.path(), &entries);

    nrs()
        .args(["claude", "observe", "--transcript"])
        .arg(&transcript)
        .arg("--dir")
        .arg(&project)
        .assert()
        .success();

    let gaps = std::fs::read_to_string(project.join("nrs.gaps.md")).unwrap();
    assert!(gaps.contains("observed:excessive-reads"));
    assert!(gaps.contains("observed:no-context"));
}

#[test]
fn observe_hook_mode_accepts_session_end_json() {
    // SessionEnd provides `transcript_path` (not `agent_transcript_path` like SubagentStop).
    // observe should fall back correctly.
    let tmp = tempfile::TempDir::new().unwrap();
    let project = tmp.path().join("project");
    let src = project.join("src/billing");
    std::fs::create_dir_all(&src).unwrap();
    for i in 0..6 {
        std::fs::write(src.join(format!("f{}.rs", i)), "").unwrap();
    }

    let entries: Vec<serde_json::Value> = (0..6)
        .map(|i| {
            let abs = project.join(format!("src/billing/f{}.rs", i));
            tool_use("Read", abs.to_str().unwrap())
        })
        .collect();
    let transcript = write_transcript(tmp.path(), &entries);

    // SessionEnd-style JSON: transcript_path + cwd
    let input = serde_json::json!({
        "transcript_path": transcript.to_str().unwrap(),
        "cwd": project.to_str().unwrap()
    });

    nrs()
        .args(["claude", "observe", "--hook-mode"])
        .write_stdin(serde_json::to_string(&input).unwrap())
        .assert()
        .success();

    let gaps = std::fs::read_to_string(project.join("nrs.gaps.md")).unwrap();
    assert!(gaps.contains("observed:excessive-reads"));
}

#[test]
fn observe_requires_transcript_or_hook_mode() {
    nrs()
        .args(["claude", "observe", "--dir", "."])
        .assert()
        .failure()
        .stderr(contains("--transcript is required"));
}
