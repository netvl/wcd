use std::process::Command;
use std::fmt::Write;
use std::ffi::OsStr;

use lazy_scoped::Lazy;

use crate::daemon::processor::playlist::Image;
use crate::common::config::ValidatedPlaylist;

pub struct ChangeCommand<'a> {
    name: &'a str,
    args: &'a [String]
}

impl<'a> ChangeCommand<'a> {
    pub fn from_config(config: &'a ValidatedPlaylist) -> ChangeCommand<'a> {
        ChangeCommand {
            name: &config.command,
            args: &config.command_args
        }
    }

    pub fn execute(&self, image: &Image) -> bool {
        // race condition, but whatever
        if !image.check() {
            return false;
        }

        let args: Vec<&OsStr> = self.args.iter()
            .map(|a| if a == "{}" { image.path.as_os_str() } else { a.as_ref() })
            .collect();

        let command_str = Lazy::new(|| {
            let mut result_str = if self.name.contains(char::is_whitespace) {
                format!("\"{}\"", self.name)
            } else {
                self.name.to_owned()
            };

            for a in &args {
                let s = a.to_string_lossy();
                let _ = if s.contains(char::is_whitespace) {
                    write!(&mut result_str, " \"{}\"", a.to_string_lossy())
                } else {
                    write!(&mut result_str, " {}", a.to_string_lossy())
                };
            }

            result_str
        });

        debug!("Executing command: {}", *command_str);

        let status = Command::new(&self.name).args(&args).status();
        match status {
            Ok(ref status) if status.success() => {},
            status => {
                match status {
                    Ok(status) => match status.code() {
                        Some(code) =>
                            warn!("Command '{}' has exited with code {}", *command_str, code),
                        None =>
                            warn!("Command '{}' has exited without status code", *command_str),
                    },
                    Err(e) => warn!("Failed to start command '{}': {}", *command_str, e),
                }
            }
        }

        true
    }
}
