use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::time::Instant;

//For more generic, integration like tests
#[test]
fn test_cli_fails_on_invalid_velocity() {
    let mut cmd = Command::cargo_bin("rsnake").unwrap();

    cmd.arg("--speed")
        .arg("super_sonic") // invalide
        .arg("--life")
        .arg("3")
        .arg("--snake-length")
        .arg("5")
        .arg("--head-symbol")
        .arg("🐍")
        .arg("--body-symbol")
        .arg("•")
        .arg("--nb-of-fruit")
        .arg("4");

    cmd.assert()
        .failure()
        .stderr(contains("error").and(contains("super_sonic")));
}
/// Ignore as fail on no TTY (as on github action) because of raw terminal mode
/// can be run locally with cargo test -- --ignored
#[test]
#[ignore]
fn test_cli_runs_with_default_args() {
    let now = Instant::now();
    let mut cmd = Command::cargo_bin("rsnake").unwrap(); // "snake" see in Cargo.toml [[bin]]
    cmd.arg("--speed")
        .arg("fast")
        .arg("--life")
        .arg("3")
        .arg("--snake-length")
        .arg("5")
        .arg("--head-symbol")
        .arg("🐍")
        .arg("--body-symbol")
        .arg("•")
        .arg("--nb-of-fruit")
        .arg("4")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .stdout(contains("Have a good game !"))
        .stderr(contains("").or(contains("No such device or address"))); // timeout 10s, or for github action

    let elapsed = now.elapsed();

    assert!(
        elapsed.as_secs() >= 10,
        "Command goes wrong, it takes : {:?}s",
        elapsed
    );
    //alt: add a test mode option, to have no user interaction
    //or simulate one
}
