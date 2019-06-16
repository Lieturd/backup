extern crate clap;

use clap::{Arg, App};

mod configuration;
mod client;
mod file_scanner;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[cfg(target_os = "macos")]
static DEFAULT_CONFIG: &'static str = "${HOME}/.config/backup.yml";
#[cfg(target_os = "linux")]
static DEFAULT_CONFIG: &'static str = "${HOME}/.config/backup.yml";
#[cfg(target_os = "windows")]
static DEFAULT_CONFIG: &'static str = "${APPDATA}\\Lieturd Backup\\backup.yml";

fn main() {
    let settings = App::new("Backup")
        .version(VERSION)
        .about("CLI utility to interface with the Lieturd Backup system")
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("CONFIG FILE")
            .help("Override the config path.")
            .takes_value(true))
        .get_matches();

    let default_config_path = shellexpand::env(DEFAULT_CONFIG).unwrap();
    let config_path = settings.value_of("config").unwrap_or(&default_config_path);

    println!("backup-cli v{} using backuplib v{}", VERSION, backuplib::VERSION);
    println!("Config file: {}", config_path);

    let config;
    match configuration::read_config(config_path) {
        Ok(entry) => {
            config = entry;
        },
        Err(err) => {
            println!("Failed to read config {}: {}", config_path, err);
            println!("Try copying the config-example.yml to the location above");
            return;
        }
    }

    println!("{:?}", config);

    let mut _client = client::Client::new(config);
    _client.run();
}
