use std::borrow::Cow;
use std::path::Path;
use std::io::{self, Write};
use std::process::Command;
use std::fs;
use std::fmt::Write as FmtWrite;
use std::thread::{self, JoinHandle};
use std::time::Duration as StdDuration;

use chan::{Receiver, Sender};
use rand::{self, Rng};
use chrono::{DateTime, UTC};

use wcd_common::proto::{ControlRequest, ControlResponse};
use wcd_common::config::{ServerConfig, ChangeMode};

pub fn start(config: ServerConfig, 
             control_req_recv: Receiver<ControlRequest>,
             control_resp_send: Sender<ControlResponse>) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut image_files_list = build_image_files_list(&config);
        info!(
            "Scanned {} files and {} directories, found {} valid image files total",
            config.files.len(), config.directories.len(), image_files_list.len()
        );

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
                    update_image_files_list(&config, &mut image_files_list);
                    thread::sleep(StdDuration::from_secs(1));  // sleep 1 second
                },
                control_req_recv.recv() -> value => {
                    if let Some(value) = value {
                        match value {
                            ControlRequest::Ping => control_resp_send.send(ControlResponse::PingOk),
                            ControlRequest::TriggerChange => {
                                last_timestamp = past_timestamp();
                                control_resp_send.send(ControlResponse::TriggerChangeOk);
                            }
                            ControlRequest::Terminate => {
                                control_resp_send.send(ControlResponse::TerminateOk);
                                break;
                            }
                        }
                    } else {
                        warn!("Control channel closed unexpectedly, exiting the server");
                        break;
                    }
                },
            }

            let current_timestamp = UTC::now();
            let difference = current_timestamp - last_timestamp;

            if !image_files_list.is_empty() {
                // change wallpaper if time has come
                loop {
                    if difference > config.change_every() {
                        let new_index = (last_index + 1) % image_files_list.len();
                        last_index = new_index;

                        let new_path: &Path = &image_files_list[new_index];

                        if !check_file(new_path) {
                            warn!("Cannot access {}, skipping it", new_path.display());
                            continue;
                        }

                        info!("Changing wallpaper to {}", new_path.display());
                        command.execute(new_path);

                        last_timestamp = current_timestamp;
                    }
                    break;
                }
            }
        }
    })
}

fn build_image_files_list(config: &ServerConfig) -> Vec<Cow<Path>> {
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

fn update_image_files_list(config: &ServerConfig, existing_files: &mut Vec<Cow<Path>>) {
    for dir in config.directories() {
        match rescan_directory(&dir, existing_files) {
            Ok(new_paths) => {
                if !new_paths.is_empty() {
                    info!("Discovered {} more files in directory {}", new_paths.len(), dir.display());
                    existing_files.extend(new_paths);
                }
            },
            Err(e) => warn!("Error reading directory {}: {}", dir.display(), e)
        }
    }
}

fn rescan_directory(dir: &Path, existing_files: &[Cow<Path>]) -> io::Result<Vec<Cow<'static, Path>>> {
    let mut result = Vec::new();
    for entry in try!(fs::read_dir(dir)) {
        let entry = try!(entry);
        let path = entry.path();
        if !existing_files.iter().map(|p| p.as_ref()).any(|p| p == path && check_file(&p)) {
            result.push(path.into());
        }
    }
    Ok(result)
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
    fn from_config(config: &ServerConfig) -> Result<ChangeCommand, &'static str> {
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
