//! Integration tests for `nrs validate`.

mod common;

use assert_cmd::Command;
use common::{project_from_fixtures, write_file};
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::TempDir;

/// Generate CLAUDE.md so the drift validator doesn't fire on every test.
fn run_generate(dir: &std::path::Path) {
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["generate", "claude", "--dir"])
        .arg(dir)
        .assert()
        .success();
}

#[test]
fn validate_clean_project_passes() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "project.context.md",
        "# Project\n\nA clean project map.\n\n## Commands\n\n- `make build` — build\n",
    );
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("ok"));
}

#[test]
fn validate_detects_source_path_in_domain_context() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "src/billing/domain.context.md",
        "# Billing\n\nSee `src/billing/service.ts` for details.\n",
    );
    write_file(tmp.path(), "project.context.md", "# Project\n");
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(contains("source file path"))
        .stdout(contains("src/billing/service.ts"));
}

#[test]
fn validate_detects_implementation_marker_in_domain_context() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "src/billing/domain.context.md",
        "# Billing\n\nWe use Prisma for data access.\n",
    );
    write_file(tmp.path(), "project.context.md", "# Project\n");
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(contains("implementation detail"))
        .stdout(contains("prisma"));
}

#[test]
fn validate_detects_broken_link_in_project_context() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "project.context.md",
        "# Project\n\n- [Testing](docs/testing.md)\n",
    );
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(contains("broken link"))
        .stdout(contains("docs/testing.md"));
}

#[test]
fn validate_detects_drift_when_generated_file_missing() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "project.context.md", "# Project\n");
    // Skip generate — CLAUDE.md never created.

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(contains("missing"));
}

#[test]
fn validate_detects_drift_when_generated_file_stale() {
    let tmp = TempDir::new().unwrap();
    write_file(tmp.path(), "project.context.md", "# Project\n");
    run_generate(tmp.path());

    // Hand-edit CLAUDE.md so it no longer matches what `generate` would produce.
    std::fs::write(tmp.path().join("CLAUDE.md"), "tampered\n").unwrap();

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(contains("out of date"));
}

#[test]
fn validate_size_warning_does_not_fail_exit_code() {
    let tmp = TempDir::new().unwrap();
    // Inner contexts have a 300-line warning limit.
    let big = "stub line\n".repeat(350);
    write_file(
        tmp.path(),
        "src/big/domain.context.md",
        &format!("# Big domain\n\n{big}"),
    );
    write_file(tmp.path(), "project.context.md", "# Project\n\n## Commands\n\n- `make build` — build\n");
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        // Warnings only — exit 0.
        .success()
        .stdout(contains("warning"))
        .stdout(contains("recommended max"));
}

#[test]
fn validate_reports_summary_counts() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "project.context.md",
        "# Project\n\n- [Missing](docs/missing.md)\n",
    );
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(contains("error(s)"));
}

// ── Duplication / similarity detection ──────────────────────────────

#[test]
fn validate_detects_exact_duplication_across_layers() {
    let tmp = project_from_fixtures(&[
        ("project-with-orders.md", "project.context.md"),
        ("orders-domain.md", "src/orders/domain.context.md"),
    ]);

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success() // duplication is a warning, not an error
        .stdout(contains("duplicated content block"));
}

#[test]
fn validate_detects_paraphrased_business_rules_across_layers() {
    let tmp = project_from_fixtures(&[
        ("project-with-billing.md", "project.context.md"),
        ("billing-domain-paraphrased.md", "src/billing/domain.context.md"),
    ]);

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success() // warning only
        .stdout(contains("near-duplicate content"))
        .stdout(contains("similar"));
}

#[test]
fn validate_no_false_positive_on_distinct_domains() {
    let tmp = project_from_fixtures(&[
        ("project-ecommerce.md", "project.context.md"),
        ("auth-domain.md", "src/auth/domain.context.md"),
        ("shipping-domain.md", "src/shipping/domain.context.md"),
    ]);

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("ok"));
}

#[test]
fn validate_strict_mode_fails_on_duplication_warning() {
    let tmp = project_from_fixtures(&[
        ("project-with-orders.md", "project.context.md"),
        ("orders-domain.md", "src/orders/domain.context.md"),
    ]);

    // Without --strict: exit 0 (warning only)
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();

    // With --strict: exit 1 (warnings become failures)
    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--strict", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(contains("duplicated content block"));
}

// ── Link validation across all layers ─────────────────────────────

#[test]
fn validate_detects_broken_link_in_domain_context() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "src/billing/domain.context.md",
        "# Billing\n\n- [Process](docs/billing-process.md)\n",
    );
    write_file(tmp.path(), "project.context.md", "# Project\n");
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(contains("broken link"))
        .stdout(contains("docs/billing-process.md"));
}

#[test]
fn validate_valid_link_in_domain_context_passes() {
    let tmp = TempDir::new().unwrap();
    let docs_dir = tmp.path().join("src/billing/docs");
    std::fs::create_dir_all(&docs_dir).unwrap();
    std::fs::write(docs_dir.join("billing.md"), "# Billing").unwrap();
    write_file(
        tmp.path(),
        "src/billing/domain.context.md",
        "# Billing\n\n- [Billing](docs/billing.md)\n",
    );
    write_file(
        tmp.path(),
        "project.context.md",
        "# Project\n\n## Commands\n\n- `make build`\n",
    );
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("ok"));
}

// ── Source path detection in markdown links ────────────────────────

#[test]
fn validate_detects_source_markdown_link_in_domain_context() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "src/billing/domain.context.md",
        "# Billing\n\nSee [service](src/billing/service.ts) for details.\n",
    );
    write_file(tmp.path(), "project.context.md", "# Project\n");
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .failure()
        .stdout(contains("source file path"))
        .stdout(contains("src/billing/service.ts"));
}

#[test]
fn validate_allows_source_markdown_link_in_implementation_context() {
    let tmp = TempDir::new().unwrap();
    write_file(
        tmp.path(),
        "src/billing/implementation.context.md",
        "# Billing Impl\n\nSee [service](src/billing/service.ts) for details.\n",
    );
    write_file(
        tmp.path(),
        "project.context.md",
        "# Project\n\n## Commands\n\n- `make build`\n",
    );
    run_generate(tmp.path());

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success();
}

// ── Duplication / similarity detection ────────────────────────────

#[test]
fn validate_detects_similarity_across_three_layers() {
    let tmp = project_from_fixtures(&[
        ("project-minimal.md", "project.context.md"),
        ("orders-cancellation.md", "src/orders/domain.context.md"),
        ("support-cancellation-similar.md", "src/support/domain.context.md"),
    ]);

    Command::cargo_bin("nrs")
        .unwrap()
        .args(["validate", "--dir"])
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(contains("similar").or(contains("duplicated")));
}
