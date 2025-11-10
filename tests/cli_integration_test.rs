use assert_cmd::{cargo_bin, Command};
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::time::Instant;

//For more generic, integration like tests
#[test]
fn cli_integration_test_fails_on_invalid_velocity() {
    let path = cargo_bin!("rsnake");
    let mut cmd = Command::new(path);
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
#[test]
#[ignore = "Fail on no TTY (as on github action) because of raw terminal mode, run it locally with cargo test -- --ignored"]
fn cli_integration_test_working_default_args() {
    let now = Instant::now();
    let path = cargo_bin!("rsnake");
    let mut cmd = Command::new(path); // "snake" see in Cargo.toml [[bin]]
    cmd.timeout(std::time::Duration::from_secs(10))
        .assert()
        .stdout(contains("Have a good"))
        .stderr(contains("").or(contains("No such device")));
    let elapsed = now.elapsed();
    assert!(
        elapsed.as_secs() >= 10,
        "Command goes wrong, it takes : {elapsed:?}s",
    );
}
#[test]
#[ignore = "Fail on no TTY (as on GitHub action) because of raw terminal mode, run it locally with cargo test -- --ignored"]
fn cli_integration_test_working_with_custom_args() {
    let now = Instant::now();
    let path = cargo_bin!("rsnake");
    let mut cmd = Command::new(path); // "snake" see in Cargo.toml [[bin]]
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
        .arg("--nb-of-fruits")
        .arg("4")
        .timeout(std::time::Duration::from_secs(10))
        .assert()
        .stdout(contains("Have a good"))
        .stderr(contains("").or(contains("No such device"))); // timeout 10s, or for github action
    let elapsed = now.elapsed();
    assert!(
        elapsed.as_secs() >= 10,
        "Command goes wrong, it takes : {elapsed:?}s",
    );
    //alt: add a test mode option to have no user interaction
    //or simulate one
}
