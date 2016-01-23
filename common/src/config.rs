use std::io::{self, Read};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::fmt::Write;
use std::borrow::Cow;
use std::collections::HashMap;

use toml::{self, DecodeError, ParserError};
use rustc_serialize::{Decodable, Decoder};
use chrono::Duration;

use util;

quick_error! {
    #[derive(Debug)]
    pub enum ConfigError {
        Io(err: io::Error) {
            from()
            description("I/O error")
            display("I/O error: {}", err)
            cause(err)
        }
        Decode(err: DecodeError) {
            from()
            description("TOML decoding error")
            display("TOML decoding error: {}", err)
            cause(err)
        }
        Parse(errs: Vec<ParserError>) {
            from()
            description("TOML parse error")
            display("TOML parse error:\n{}", errs.iter().fold(String::new(), |mut s, e| {
                let _ = write!(&mut s, "* {}\n", e);
                s
            }))
            cause(&errs[0])
        }
        Validation(msg: Cow<'static, str>) {
            from()
            from(s: String) -> (s.into())
            from(s: &'static str) -> (s.into())
            description("validation error")
            display("validation error: {}", msg)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchMode {
    Disabled,
    Poll(Duration),
}

impl Decodable for WatchMode {
    fn decode<D: Decoder>(d: &mut D) -> Result<WatchMode, D::Error> {
        d.read_str().and_then(|s| match s.to_lowercase().as_ref() {
            "disabled" => Ok(WatchMode::Disabled),
            other => match util::parse_duration(&other) {
                Some(d) => Ok(WatchMode::Poll(d)),
                None => Err(d.error(&format!("invalid watch value: {}", other)))
            }
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChangeMode {
    Sequential,
    Random
}

impl Decodable for ChangeMode {
    fn decode<D: Decoder>(d: &mut D) -> Result<ChangeMode, D::Error> {
        d.read_str().and_then(|s| match s.to_lowercase().as_ref() {
            "sequential" => Ok(ChangeMode::Sequential),
            "random" => Ok(ChangeMode::Random),
            something_else => Err(d.error(&format!("invalid change mode: {}", something_else)))
        })
    }
}

// Configuration directly corresponding to the one stored in file

#[derive(RustcDecodable, Debug, Clone)]
pub struct Config {
    pub common: CommonConfig,
    pub server: ServerConfig,
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct CommonConfig {
    pub endpoint: String
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct ServerConfig {
    pub default_playlist: String,
    pub watch: Option<WatchMode>,
    pub defaults: Option<Defaults>,
    pub playlists: HashMap<String, Playlist>,
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct Playlist {
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub command: Option<Vec<String>>,
    pub mode: Option<ChangeMode>,
    pub change_every: Option<ParsedDuration>,
    pub trigger_on_select: Option<bool>,
    pub use_last_on_select: Option<bool>,
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct Defaults {
    pub command: Option<Vec<String>>,
    pub mode: Option<ChangeMode>,
    pub change_every: Option<ParsedDuration>,
    pub trigger_on_select: Option<bool>,
    pub use_last_on_select: Option<bool>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ParsedDuration(Duration);

// Configuration after validation and defaults resolution step

#[derive(Debug, Clone)]
pub struct ValidatedConfig {
    pub common: CommonConfig,
    pub server: ValidatedServerConfig
}

#[derive(Debug, Clone)]
pub struct ValidatedServerConfig {
    pub default_playlist: String,
    pub watch: WatchMode,
    pub playlists: HashMap<String, ValidatedPlaylist>,
}

#[derive(Debug, Clone)]
pub struct ValidatedPlaylist {
    pub files: Vec<PathBuf>,
    pub directories: Vec<PathBuf>,
    pub command: String,
    pub command_args: Vec<String>,
    pub mode: ChangeMode,
    pub change_every: Duration,
    pub trigger_on_select: bool,
    pub use_last_on_select: bool,
}

impl Decodable for ParsedDuration {
    fn decode<D: Decoder>(d: &mut D) -> Result<ParsedDuration, D::Error> {
        let s = try!(d.read_str());
        util::parse_duration(&s)
            .map(ParsedDuration)
            .ok_or_else(|| d.error(&format!("invalid duration format: {}", s)))
    }
}

pub fn load(path: &Path) -> Result<ValidatedConfig, ConfigError> {
    let mut file = try!(File::open(path));
    let mut data = String::new();
    let _ = try!(file.read_to_string(&mut data));

    let mut parser = toml::Parser::new(&data);
    let config_value = match parser.parse() {
        Some(config) => config,
        None => return Err(parser.errors.into())
    };

    let config_value = toml::Value::Table(config_value);

    Config::decode(&mut toml::Decoder::new(config_value))
        .map_err(From::from)
        .and_then(validate)
}

fn validate(config: Config) -> Result<ValidatedConfig, ConfigError> {
    let Config { 
        server: ServerConfig { default_playlist, watch, defaults, playlists }, 
        common
    } = config;

    if !playlists.contains_key(&default_playlist) {
        return Err(format!("unknown playlist name {} configured as a default playlist", default_playlist).into());
    }

    fn check_command(cmd: &[String], playlist: Option<&str>) -> Result<(), ConfigError> {
        if cmd.is_empty() {
            Err(if let Some(playlist) = playlist {
                format!("empty command is configured in playlist {}", playlist).into()
            } else {
                "empty default command is configured".into()
            })
        } else if cmd.iter().all(|p| p != "{}") {
            Err(if let Some(playlist) = playlist {
                format!("configured command in playlist {} has no file placeholder in it", playlist).into()
            } else {
                "configured default command has no file placeholder in it".into()
            })
        } else {
            Ok(())
        }
    }

    let defaults = defaults.as_ref();

    if let Some(cmd) = defaults.and_then(|d| d.command.as_ref()) {
        try!(check_command(cmd, None));
    }

    let mut validated_playlists = HashMap::new();
    for (name, playlist) in playlists {
        let files = playlist.files.iter()
            .map(util::str_to_path_0)
            .map(|p| p.into_owned())
            .collect();

        let directories = playlist.directories.iter()
            .map(util::str_to_path_0)
            .map(|p| p.into_owned())
            .collect();

        let (command, command_args) = match playlist.command.or_else(|| defaults.and_then(|d| d.command.clone())) {
            Some(mut full_command) => {
                try!(check_command(&full_command, Some(&name)));
                let command = full_command.remove(0);  // full_command is checked to be non-empty
                (command, full_command)  // full_command now only contains args
            }
            None => return Err(format!("playlist {} has no command configured and no default is set", name).into())
        };

        let mode = match playlist.mode.or(defaults.and_then(|d| d.mode)) {
            Some(mode) => mode,
            None => return Err(format!("playlist {} has no change mode configured and no default is set", name).into())
        };

        let change_every = match playlist.change_every.or_else(|| defaults.and_then(|d| d.change_every.clone())) {
            Some(change_every) => change_every,
            None => return Err(format!("playlist {} has no change interval configured and no default is set", name).into())
        };

        let trigger_on_select = playlist.trigger_on_select
            .or(defaults.and_then(|d| d.trigger_on_select))
            .unwrap_or(true);

        let use_last_on_select = playlist.use_last_on_select
            .or(defaults.and_then(|d| d.use_last_on_select))
            .unwrap_or(true);

        validated_playlists.insert(name, ValidatedPlaylist {
            files: files,
            directories: directories,
            command: command,
            command_args: command_args,
            mode: mode,
            change_every: change_every.0,
            trigger_on_select: trigger_on_select,
            use_last_on_select: use_last_on_select
        });
    }

    Ok(ValidatedConfig {
        common: common,
        server: ValidatedServerConfig {
            default_playlist: default_playlist,
            watch: watch.unwrap_or(WatchMode::Poll(Duration::seconds(10))),
            playlists: validated_playlists
        }
    })
}

