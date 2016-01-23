use std::io::{self, Read};
use std::fs::File;
use std::path::Path;
use std::fmt::Write;
use std::iter;
use std::slice;
use std::borrow::Cow;

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
        Other(msg: Cow<'static, str>) {
            from()
            from(s: String) -> (s.into())
            from(s: &'static str) -> (s.into())
            description(&msg)
            display("{}", msg)
        }
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

#[derive(RustcDecodable, Debug, Clone)]
pub struct CommonConfig {
    pub endpoint: String
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct ServerConfig {
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub command: Vec<String>,
    pub mode: ChangeMode,
    pub change_every: String
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct Config {
    pub common: CommonConfig,
    pub server: ServerConfig,
}

impl ServerConfig {
    pub fn files(&self) -> iter::Map<slice::Iter<String>, fn(&String) -> Cow<Path>> {
        self.files.iter().map(util::str_to_path_0)
    }

    pub fn directories(&self) -> iter::Map<slice::Iter<String>, fn(&String) -> Cow<Path>> {
        self.directories.iter().map(util::str_to_path_0)
    }

    pub fn change_every(&self) -> Duration {
        util::parse_duration(&self.change_every).unwrap()
    }
}

pub fn load(path: &Path) -> Result<Config, ConfigError> {
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
        .and_then(|config| {
            match util::parse_duration(&config.server.change_every) {
                Some(_) => Ok(config),
                None => Err(format!("invalid duration format: {}", config.server.change_every).into())
            }
        })
}

