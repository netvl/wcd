#[macro_use] extern crate log;
#[macro_use] extern crate chan;
extern crate env_logger;
#[macro_use(crate_version)]
extern crate clap;
extern crate chrono;
extern crate rand;
extern crate nanomsg;
extern crate wcd_common;

use std::borrow::Cow;
use std::path::Path;
use std::io::{self, Write};
use std::process;

use clap::{App, AppSettings, Arg};

use wcd_common::config;
use wcd_common::util;

mod server;
mod control;

fn main() {
    env_logger::init().unwrap();

    let matches = App::new("wallpaper change daemon")
        .version(crate_version!())
        .author("Vladimir Matveev <vladimir.matweev@gmail.com>")
        .about("Changes the wallpaper in a timely manner using playlists.")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .arg(
            Arg::from_usage("-c, --config=[FILE] 'Path to the configuration file'")
                .default_value("~/.config/wcd/config.toml")
        )
        .get_matches();

    let config_path = matches.value_of("config").unwrap();
    let config_path: Cow<Path> = util::str_to_path(config_path);

    let config = config::load(&config_path).unwrap_or_else(|e| {
        let _ = writeln!(
            &mut io::stderr(), "Error loading configuration from {}: {}", 
            config_path.display(), e
        );
        process::exit(1);
    });

    debug!("Loaded configuration: {:#?}", config);

    let endpoint = config.common.endpoint;
    let config = config.server;

    let (control_req_recv, control_resp_send, control_handle) = control::start(endpoint);
    let server_handle = server::start(config, control_req_recv, control_resp_send);

    let _ = control_handle.join();
    let _ = server_handle.join();
}

