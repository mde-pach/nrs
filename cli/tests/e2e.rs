//! End-to-end happy path: init → write context → generate → validate.

mod common;

use assert_cmd::Command;
use common::{git_repo, write_file};

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
