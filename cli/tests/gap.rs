//! Integration tests for `nrs gap report` and `nrs gap summary`.

use assert_cmd::Command;
use predicates::str::contains;

fn nrs() -> Command {
    Command::cargo_bin("nrs").unwrap()
}

#[test]
fn report_creates_gaps_file_with_header() {
    let tmp = tempfile::TempDir::new().unwrap();

    nrs()
        .args([
            "gap", "report",
            "--type", "missing-context",
            "--target", "src/billing/",
            "--description", "no context file for billing domain",
            "--dir",
        ])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("reported missing-context gap for src/billing/"));

    let content = std::fs::read_to_string(tmp.path().join("nrs.gaps.md")).unwrap();
    assert!(content.contains("# NRS Gaps"), "should have header");
    assert!(content.contains("| Type | Target | Description | Source | Confidence |"), "should have table header");
    assert!(content.contains("| missing-context | src/billing/ | no context file for billing domain | manual | - |"));
}

#[test]
fn report_appends_to_existing_file() {
    let tmp = tempfile::TempDir::new().unwrap();

    // First report
    nrs()
        .args([
            "gap", "report",
            "--type", "missing-context",
            "--target", "src/billing/",
            "--description", "first gap",
            "--dir",
        ])
        .arg(tmp.path())
        .assert()
        .success();

    // Second report
    nrs()
        .args([
            "gap", "report",
            "--type", "wrong",
            "--target", "src/orders/",
            "--description", "second gap",
            "--dir",
        ])
        .arg(tmp.path())
        .assert()
        .success();

    let content = std::fs::read_to_string(tmp.path().join("nrs.gaps.md")).unwrap();
    // Header appears only once
    assert_eq!(content.matches("# NRS Gaps").count(), 1);
    assert_eq!(content.matches("| Type | Target | Description | Source | Confidence |").count(), 1);
    // Both rows present
    assert!(content.contains("| missing-context | src/billing/ | first gap | manual | - |"));
    assert!(content.contains("| wrong | src/orders/ | second gap | manual | - |"));
}

#[test]
fn report_preserves_duplicates() {
    let tmp = tempfile::TempDir::new().unwrap();

    for _ in 0..3 {
        nrs()
            .args([
                "gap", "report",
                "--type", "missing-pattern",
                "--target", "src/billing/services/",
                "--description", "retry strategy not documented",
                "--dir",
            ])
            .arg(tmp.path())
            .assert()
            .success();
    }

    let content = std::fs::read_to_string(tmp.path().join("nrs.gaps.md")).unwrap();
    assert_eq!(
        content.matches("| missing-pattern | src/billing/services/ | retry strategy not documented | manual | - |").count(),
        3,
        "all three duplicate rows should be preserved"
    );
}

#[test]
fn report_invalid_type_errors() {
    let tmp = tempfile::TempDir::new().unwrap();

    nrs()
        .args([
            "gap", "report",
            "--type", "invalid-type",
            "--target", "src/billing/",
            "--description", "some gap",
            "--dir",
        ])
        .arg(tmp.path())
        .assert()
        .failure()
        .stderr(contains("invalid gap type 'invalid-type'"))
        .stderr(contains("missing-context"))
        .stderr(contains("missing-concept"))
        .stderr(contains("missing-pattern"))
        .stderr(contains("wrong"));
}

#[test]
fn report_empty_description_errors() {
    let tmp = tempfile::TempDir::new().unwrap();

    nrs()
        .args([
            "gap", "report",
            "--type", "wrong",
            "--target", "src/billing/",
            "--description", "",
            "--dir",
        ])
        .arg(tmp.path())
        .assert()
        .failure()
        .stderr(contains("--description must not be empty"));
}

#[test]
fn report_empty_target_errors() {
    let tmp = tempfile::TempDir::new().unwrap();

    nrs()
        .args([
            "gap", "report",
            "--type", "wrong",
            "--target", "",
            "--description", "something",
            "--dir",
        ])
        .arg(tmp.path())
        .assert()
        .failure()
        .stderr(contains("--target must not be empty"));
}

#[test]
fn summary_no_gaps_file() {
    let tmp = tempfile::TempDir::new().unwrap();

    nrs()
        .args(["gap", "summary", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("no gaps reported"));
}

#[test]
fn summary_groups_by_target() {
    let tmp = tempfile::TempDir::new().unwrap();

    // Report gaps for two different targets
    for (gap_type, target, desc) in [
        ("missing-context", "src/billing/", "no context file"),
        ("wrong", "src/orders/", "pricing rules outdated"),
        ("missing-pattern", "src/billing/", "retry strategy not documented"),
    ] {
        nrs()
            .args([
                "gap", "report",
                "--type", gap_type,
                "--target", target,
                "--description", desc,
                "--dir",
            ])
            .arg(tmp.path())
            .assert()
            .success();
    }

    nrs()
        .args(["gap", "summary", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("src/billing/ (2 gaps)"))
        .stdout(contains("[missing-context] no context file"))
        .stdout(contains("[missing-pattern] retry strategy not documented"))
        .stdout(contains("src/orders/ (1 gap)"))
        .stdout(contains("[wrong] pricing rules outdated"))
        .stdout(contains("3 total gap(s)"));
}

#[test]
fn summary_shows_source_pattern_for_observed_gaps() {
    let tmp = tempfile::TempDir::new().unwrap();

    // Manually write a gaps file mixing a manual and an observed gap.
    let content = "# NRS Gaps\n\n\
| Type | Target | Description | Source | Confidence |\n\
|------|--------|-------------|--------|------------|\n\
| missing-context | src/billing/ | manual entry | manual | - |\n\
| missing-pattern | src/billing/ | agent re-read | observed:re-reads | medium |\n";
    std::fs::write(tmp.path().join("nrs.gaps.md"), content).unwrap();

    nrs()
        .args(["gap", "summary", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        // Manual gap: no source shown
        .stdout(contains("[missing-context] manual entry"))
        // Observed gap: source shown in parentheses
        .stdout(contains("[missing-pattern] (observed:re-reads) agent re-read"));
}

#[test]
fn summary_shows_total_count() {
    let tmp = tempfile::TempDir::new().unwrap();

    for i in 0..5 {
        nrs()
            .args([
                "gap", "report",
                "--type", "missing-pattern",
                "--target", &format!("src/domain-{}/", i),
                "--description", "some gap",
                "--dir",
            ])
            .arg(tmp.path())
            .assert()
            .success();
    }

    nrs()
        .args(["gap", "summary", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("5 total gap(s)"));
}
