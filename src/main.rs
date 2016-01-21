#[macro_use] extern crate quick_error;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate toml;
extern crate rustc_serialize;
extern crate docopt;
extern crate chrono;
extern crate rand;
extern crate chan_signal;
#[macro_use] extern crate chan;

use std::borrow::Cow;
use std::path::Path;
use std::io::{self, Write};
use std::process::{self, Command};
use std::fs;
use std::fmt::Write as FmtWrite;
use std::thread;
use std::time::Duration as StdDuration;

use docopt::Docopt;
use chrono::{DateTime, UTC};
use rand::Rng;
use chan_signal::Signal;

use config::{Config, ChangeMode};

mod config;
mod util;

const USAGE: &'static str = r"
Usage: wchangerd [options]

Options:
    -c FILE, --config FILE  path to configuration file [default: ~/.config/wchangerd/config.toml]
    -h, --help              show this message
    -v, --version           show version information
";

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    // prepare sighup listener
    let signal = chan_signal::notify(&[Signal::HUP, Signal::INT, Signal::TERM]);

    env_logger::init().unwrap();

    let args = Docopt::new(USAGE)
        .unwrap_or_else(|e| e.exit())
        .help(true)
        .version(Some(VERSION.into()))
        .parse()
        .unwrap_or_else(|e| e.exit());

    let config_path = args.get_str("--config");
    let config_path: Cow<Path> = util::str_to_path(config_path);

    let config = config::load(&config_path).unwrap_or_else(|e| {
        let _ = writeln!(
            &mut io::stderr(), "Error loading configuration from {}: {}", 
            config_path.display(), e
        );
        process::exit(1);
    });

    debug!("Loaded configuration: {:#?}", config);

    let mut image_files_list = build_image_files_list(&config);
    info!(
        "Scanned {} files and {} directories, found {} valid image files total",
        config.files.len(), config.directories.len(), image_files_list.len()
    );

    if image_files_list.is_empty() {
        info!("No images found, exiting");
        return;
    }

    // shuffle the list if necessary
    if let ChangeMode::Random = config.mode {
        rand::thread_rng().shuffle(&mut image_files_list);
    }

    // prepare the wall changing command
    let command = match ChangeCommand::from_config(&config) {
        Ok(command) => command,
        Err(msg) => {
            error!("Error preparing command: {}", msg);
            return;
        }
    };

    fn past_timestamp() -> DateTime<UTC> {
        DateTime::parse_from_str("0+0000", "%s%z")
                .unwrap()
                .with_timezone(&UTC)
    }

    let mut last_timestamp = past_timestamp();
    let mut last_index = image_files_list.len()-1;
    loop {
        chan_select! {
            default => {
                thread::sleep(StdDuration::from_secs(1));  // sleep 1 second
            },
            signal.recv() -> value => {
                match value {
                    Some(Signal::HUP) => {
                        // reset the timestamp to force the next change
                        last_timestamp = past_timestamp();
                    },
                    _ => {
                        break;
                    }
                }
            },
        }

        let current_timestamp = UTC::now();
        let difference = current_timestamp - last_timestamp;

        // change wallpaper if time has come
        if difference > config.change_every() {
            let new_index = (last_index + 1) % image_files_list.len();
            info!("Changing wallpaper to {}", image_files_list[new_index].display());

            command.execute(&image_files_list[new_index]);
            last_index = new_index;
            last_timestamp = current_timestamp;
        }
    }
}

fn build_image_files_list(config: &Config) -> Vec<Cow<Path>> {
    let mut files: Vec<_> = Vec::new();

    for file in config.files() {
        if check_file(&file) {
            files.push(file.into());
        }
    }
    
    for dir in config.directories() {
        if let Err(e) = scan_directory(&dir, &mut files) {
            warn!("Error reading directory {}: {}", dir.display(), e);
        }
    }

    files
}

fn scan_directory(dir: &Path, files: &mut Vec<Cow<Path>>) -> io::Result<()> {
    for entry in try!(fs::read_dir(dir)) {
        let entry = try!(entry);
        let path = entry.path();
        if check_file(&path) {
            files.push(path.into());
        }
    }
    Ok(())
}

fn check_file(file: &Path) -> bool {
    if check_file_format(file) {
        match check_file_access(file) {
            Ok(r) => r,
            Err(e) => {
                warn!("Error accessing file {}: {}", file.display(), e);
                false
            }
        }
    } else {
        false
    }
}

fn check_file_format(file: &Path) -> bool {
    if let Some(ext) = file.extension() {
        if let Some(ext) = ext.to_str() {
            match &*ext.to_lowercase() {
                "jpg" | "jpeg" | "png" | "gif" => return true,
                _ => {}
            }
        }
    }
    false
}

fn check_file_access(file: &Path) -> io::Result<bool> {
    fs::metadata(file).map(|m| m.is_file())
}

struct ChangeCommand {
    name: String,
    args: Vec<String>
}

impl ChangeCommand {
    fn from_config(config: &Config) -> Result<ChangeCommand, &'static str> {
        if config.command.is_empty() {
            Err("command is not configured")
        } else {
            let mut parts = config.command.iter();
            let name = parts.next().unwrap().clone();  // won't fail
            let args: Vec<_> = parts.map(|s| s.clone()).collect();
            if args.iter().any(|a| a == "{}") {
                Ok(ChangeCommand {
                    name: name,
                    args: args
                })
            } else {
                Err("none of command arguments are a placeholder")
            }
        }
    }

    fn execute(&self, file: &Path) {
        let args: Vec<&str> = self.args.iter()
            .filter_map(|a| if a == "{}" { file.to_str() } else { Some(a) })
            .collect();
        debug!("Executing command {} with arguments: {:?}", self.name, args);
        let status = Command::new(&self.name).args(&args).status();
        match status {
            Ok(ref status) if status.success() => {},
            status => {
                let mut args_str = String::new();
                for a in args.iter() {
                    let _ = write!(&mut args_str, "\"{}\"", a);
                }
                let command_str = format!("\"{}\" {}", self.name, args_str);

                match status {
                    Ok(status) => match status.code() {
                        Some(code) =>
                            warn!("command {} has exited with code {}", command_str, code),
                        None =>
                            warn!("command {} has exited without status code", command_str)
                    },
                    Err(e) =>
                        warn!("failed to start command {}: {}", command_str, e)
                }
            }
        }
    }
}
