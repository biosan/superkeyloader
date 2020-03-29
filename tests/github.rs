#[macro_use]
pub extern crate log;

#[cfg(test)]
mod github_integration_test {

    const CLI_BIN: &str = "superkeyloader"; // Binary file name
    const VALID_USERNAME: &str = "biosan"; // My own username
    const VALID_USERNAME_KEYS: usize = 3; // I currently have 3 SSH keys TODO: Use a more robust solution
    const INVALID_USERNAME: &str = "test-"; // It ends with a hyphen
    const MISSING_USERNAME: &str = "about"; // It's a reserved username so it will never be used
    const GITHUB_TOKEN: &str = "GITHUB_TOKEN";

    use assert_cmd::Command;
    use predicates::prelude::*; // Used for writing assertions
    use rand::Rng;
    use std::env;
    use std::fs::File;
    use std::io::{BufRead, BufReader, Write};
    use std::path::PathBuf;
    // NOTE: Switched to a random file into OS's temporary directory.
    //use tempfile::NamedTempFile;

    #[test]
    fn invalid_username() -> Result<(), Box<dyn std::error::Error>> {
        init();
        let mut cmd = Command::cargo_bin(CLI_BIN)?;
        cmd = _add_api_token(cmd); // Set to token to raise API rate limit
        cmd = _env_args(cmd); // Add additional arguments from 'RUST_TEST_ARGS' environment variable
        cmd.arg(INVALID_USERNAME);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Invalid username")); // TODO: Case insensitive match
        Ok(())
    }

    #[test]
    fn missing_username() -> Result<(), Box<dyn std::error::Error>> {
        init();
        let mut cmd = Command::cargo_bin(CLI_BIN)?;
        cmd = _add_api_token(cmd); // Set to token to raise API rate limit
        cmd = _env_args(cmd); // Add additional arguments from 'RUST_TEST_ARGS' environment variable
        cmd.arg(MISSING_USERNAME);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Wrong username")); // TODO: Case insensitive match
        Ok(())
    }

    #[test]
    fn valid_user_create_file() -> Result<(), Box<dyn std::error::Error>> {
        init();
        let file_path = _create_test_file(0);

        let mut cmd = Command::cargo_bin(CLI_BIN)?;
        cmd.arg("--human"); // Force human parsable output
        cmd.arg("--output"); // Write keys into file './test'
        cmd.arg(&file_path);
        cmd = _add_api_token(cmd); // Set to token to raise API rate limit
        cmd = _env_args(cmd); // Add additional arguments from 'RUST_TEST_ARGS' environment variable
        cmd.arg(VALID_USERNAME);
        cmd.assert().success().stdout(
            predicate::str::contains("Downloaded")
                .and(predicates::str::contains("SSH keys"))
                .and(predicates::str::contains(VALID_USERNAME)),
        ); // TODO: Case insensitive match

        let lines = _read_test_file(&file_path);

        assert_eq!(lines.len(), VALID_USERNAME_KEYS);

        Ok(())
    }

    #[test]
    fn valid_user_append_file() -> Result<(), Box<dyn std::error::Error>> {
        init();
        let exising_lines: usize = 3;

        // Create test file with 3 lines
        let file_path = _create_test_file(3);

        let mut cmd = Command::cargo_bin(CLI_BIN)?;
        cmd.arg("--human"); // Force human parsable output
        cmd.arg("--output"); // Write keys into file './test'
        cmd.arg(&file_path);
        cmd = _add_api_token(cmd); // Set to token to raise API rate limit
        cmd = _env_args(cmd); // Add additional arguments from 'RUST_TEST_ARGS' environment variable
        cmd.arg(VALID_USERNAME);
        cmd.assert().success().stdout(
            predicate::str::contains("Downloaded")
                .and(predicates::str::contains("SSH keys"))
                .and(predicates::str::contains(VALID_USERNAME)),
        ); // TODO: Case insensitive match

        let lines = _read_test_file(&file_path);

        assert_eq!(lines.len(), exising_lines + VALID_USERNAME_KEYS);

        Ok(())
    }

    #[test]
    fn valid_user_json_output() -> Result<(), Box<dyn std::error::Error>> {
        init();
        let file_path = _create_test_file(0);

        let mut cmd = Command::cargo_bin(CLI_BIN)?;
        cmd.arg("--json"); // Force human parsable output
        cmd.arg("--output"); // Write keys into file './test'
        cmd.arg(file_path);
        cmd = _add_api_token(cmd); // Set to token to raise API rate limit
        cmd = _env_args(cmd); // Add additional arguments from 'RUST_TEST_ARGS' environment variable
        cmd.arg(VALID_USERNAME);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("{\"keys\":[\"ssh-"));

        Ok(())
    }

    //
    // Utility functions
    //

    fn init() {
        let _ = pretty_env_logger::env_logger::builder()
            .is_test(true)
            .try_init();
    }

    fn _create_test_file(lines: usize) -> std::path::PathBuf {
        let postfix: u32 = rand::thread_rng().gen();
        let tempdir = std::env::temp_dir();
        let filename = PathBuf::from(format!("temp-{}", postfix));
        let filepath = tempdir.join(filename);
        let mut file = File::create(&filepath).unwrap();
        for _ in 0..lines {
            writeln!(file, "helloooo").unwrap();
        }
        filepath
    }

    fn _read_test_file(path: &PathBuf) -> Vec<String> {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().map(std::result::Result::unwrap).collect();
        lines
    }

    fn _add_api_token(mut cmd: assert_cmd::cmd::Command) -> assert_cmd::cmd::Command {
        let token = env::var(GITHUB_TOKEN);
        if let Ok(gh_token) = token {
            warn!("Got a GitHub token!");
            cmd.arg("--token");
            cmd.arg(gh_token);
        }
        cmd
    }

    // TODO: Maybe use 'RUST_LOG' env var to set only log level
    // NOTE: Log output should be written on file to not interfer with STDOUT/STDERR
    fn _env_args(mut cmd: assert_cmd::cmd::Command) -> assert_cmd::cmd::Command {
        let additional_args = env::var("RUST_TEST_ARGS");
        if let Ok(string_args) = additional_args {
            warn!("Got 'RUST_TEST_ARGS' environment variable");
            let args: Vec<&str> = string_args.split_whitespace().collect();
            cmd.args(args);
        }
        cmd
    }
}
