use std::borrow::Cow;
use std::path::Path;
use std::process;

use clap::{App, AppSettings, Arg, ArgMatches};
use log4rs;

use common::{config, util};

mod server;
mod control;
mod cache;

pub const SUBCOMMAND_NAME: &'static str = "daemon";

pub fn subcommand() -> App<'static, 'static> {
    App::new(SUBCOMMAND_NAME)
        .about("Starts the wallpaper change daemon")
        .setting(AppSettings::ColoredHelp)
}

pub fn main(config_path: Cow<Path>) {
    let config = config::load(&config_path).unwrap_or_else(|e| {
        error!("Error loading configuration from {}: {}", config_path.display(), e);
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