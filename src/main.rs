#[macro_use]
pub extern crate log;

extern crate clap_verbosity_flag;
extern crate pretty_env_logger;

use atty::Stream;
use human_panic::setup_panic;
use serde_json::json;
use shellexpand;
use std::fs::OpenOptions;
use std::io::prelude::*;
use structopt::StructOpt;

use superkeyloader_lib::*;

//
// CLI Arguments parsing struct
//
#[derive(Debug, StructOpt)]
struct CliArgs {
    // Required argument. GitHub username.
    username: String,

    // Optional output file (if you need a to append keys to a file other than
    // '~/.ssh/authorized_keys')
    #[structopt(
        short = "o",
        long = "output",
        required = false,
        default_value = "~/.ssh/authorized_keys",
        parse(from_os_str)
    )]
    path: std::path::PathBuf,

    // Optional GitHub API token (use if you reach API rate limits)
    // Acutally used only during testing on CI to overcome API rate limits
    #[structopt(long = "token")]
    token: Option<String>,

    // Enable setting verbosity level with '--verbose', '-v', '-vv', etc. flags
    #[structopt(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[structopt(short = "m", long = "human", conflicts_with_all(&["json", "stdout"]))]
    human: bool,

    #[structopt(short = "j", long = "json", conflicts_with_all(&["human", "stdout"]))]
    json: bool,

    #[structopt(short = "p", long = "stdout", conflicts_with_all(&["human", "json"]))]
    stdout: bool,
}

fn main() -> Result<(), ExitDisplay<String>> {
    // Enable human-friendly panic message
    setup_panic!();

    //
    // Parse command line args
    //
    let args = CliArgs::from_args();

    //
    // Enable STDOUT/STDERR logging with level set by environment variable,
    // or by verbosity flag
    //
    let log_level = match args.verbose.log_level() {
        Some(level) => level.to_level_filter(),
        None => log::LevelFilter::Off, // IF 'Option<Level>' it's 'None', then 'LevelFilter' is 'Off'
    };
    let pkg_name = Option::from(env!("CARGO_PKG_NAME")); // Prints only logs from this package

    // Initialize 'pretty_env_logger' with a filter.
    // Only logs from this package and log level set by flags.
    pretty_env_logger::formatted_builder()
        .filter(pkg_name, log_level)
        .init();

    info!("Human: {} - JSON: {}", &args.human, &args.json);
    //
    // Download keys and build a vector of key strings
    // and handling connection and "availability" errors
    //
    info!("Downloading keys for '{}' from GitHub...", &args.username);

    let keys = error_handler_wrapper(gh::get_keys(&args.username, args.token))?;
    let keys_number = keys.len();

    info!("Downloaded {} keys.", keys_number);

    //
    // Create 'authorized_keys' file if not exists and access it in 'append mode'.
    // (if testing, will use a local file)
    //

    let args_path_string = args.path.to_str().unwrap();

    let authorized_keys_path = shellexpand::tilde(args_path_string).to_owned().to_string();

    info!("Got 'authorized_keys' file path: {}", authorized_keys_path);

    let authorized_keys_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(authorized_keys_path)
        .unwrap();

    info!("Opened/Created 'authorized_keys' file in append mode");

    for (i, key) in keys.iter().enumerate() {
        match writeln!(&authorized_keys_file, "{}", key) {
            Ok(..) => {
                // TODO: Use something safer than substring (like a functional 'truncate').
                //       It will panics if 'key' is less than 16 chars.
                debug!("Wrote key {}/{} ({}...)", i, keys_number, &key[..48]);
            }
            Err(why) => {
                return Err(format!(
                    "Error writing key {}/{} to 'authorized_keys' file. Caused by {}",
                    i, keys_number, why
                )
                .into());
            }
        };
    }

    //
    // IF output is 'interactive' THEN prints a simple summary message.
    // IF output is 'non-interactive' THEN print a JSON that contains the downloaded keys.
    // i.e.:
    //
    // {
    //   "keys": [
    //     "ssh-rsa AAAAB3NzaC1yc2EAAAAD...",
    //     "ssh-rsa AAAAB3NzaC1yc2EAAAAD..."
    //   ]
    // }
    //
    let output: String;

    let is_tty = atty::is(Stream::Stdout);

    // Command line flags have precedence, if no flag is set, then
    //  if command is executed in an interactive terminal will output
    //  a human message, else it will output JSON
    let human_output = !args.json && is_tty || args.human;

    if human_output {
        output = format!(
            "Downloaded {} SSH keys for user '{}' \
            from {} and appended to 'authorized_keys' file.",
            keys_number, &args.username, "GitHub"
        );
    } else {
        output = json!({ "keys": keys }).to_string();
    }

    if !args.verbose.is_silent() {
        println!("{}", output);
    }

    Ok(())
}
