#[macro_use] extern crate quick_error;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate toml;
extern crate rustc_serialize;
extern crate docopt;
extern crate chrono;

use std::borrow::Cow;
use std::path::Path;
use std::io::{self, Write};
use std::process;
use std::fs;

use docopt::Docopt;

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

    let image_files_list = build_image_files_list(&config);
    debug!(
        "Scanned {} files and {} directories, found {} valid image files total",
        config.files.len(), config.directories.len(), image_files_list.len()
    );

    if image_files_list.is_empty() {
        info!("No images found, exiting");
        return;
    }

    if config.mode == ChangeMode::Fixed {
    } else {

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
