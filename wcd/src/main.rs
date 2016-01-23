#[macro_use] extern crate log;
#[macro_use] extern crate chan;
extern crate env_logger;
extern crate rustc_serialize;
extern crate docopt;
extern crate chrono;
extern crate rand;
extern crate nanomsg;
extern crate bincode;
extern crate wcd_common;

use std::borrow::Cow;
use std::path::Path;
use std::io::{self, Write};
use std::process;

use docopt::Docopt;

use wcd_common::config;
use wcd_common::util;

mod server;
mod control;

const USAGE: &'static str = r"
Usage: wcd [options]

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

    let endpoint = config.common.endpoint;
    let config = config.server;

    let (control_req_recv, control_resp_send, control_handle) = control::start(endpoint);
    let server_handle = server::start(config, control_req_recv, control_resp_send);

    let _ = control_handle.join();
    let _ = server_handle.join();
}

