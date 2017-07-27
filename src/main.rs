#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate diesel;
#[macro_use(crate_version)] extern crate clap;
extern crate chrono;
extern crate toml;
extern crate serde;
extern crate shellexpand;
extern crate log4rs;
extern crate appdirs;
extern crate rand;
extern crate lazy_scoped;
extern crate protobuf;
extern crate grpc;
extern crate futures;
extern crate futures_cpupool;
extern crate tls_api;
extern crate parking_lot;

use std::borrow::Cow;
use std::path::Path;

use clap::{App, AppSettings, Arg};

use common::log::LogLevel;

#[macro_use]
mod macros;
mod common;
mod cli;
mod daemon;

fn main() {
    let matches = App::new("wcd")
        .version(crate_version!())
        .author("Vladimir Matveev <vladimir.matweev@gmail.com>")
        .about("A wallpaper change daemon and its control utility.")
        .global_setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::from_usage("-c, --config=[FILE] 'Path to the configuration file'")
                .default_value("~/.config/wcd/config.toml")
        )
        .args_from_usage(
            "-v... 'Enable verbose output (up to two times)'"
        )
        .subcommand(daemon::subcommand())
        .subcommands(cli::subcommands())
        .get_matches();

    let log_level = match matches.occurrences_of("v") {
        0 => LogLevel::Normal,
        1 => LogLevel::Debug,
        _ => LogLevel::Trace,
    };
    common::log::configure_or_panic(log_level);

    let config_path = matches.value_of("config").unwrap();
    let config_path: Cow<Path> = common::util::str_to_path(config_path);

    match matches.subcommand() {
        (daemon::SUBCOMMAND_NAME, Some(_)) => daemon::main(config_path),
        (subcommand, Some(matches)) => cli::main(config_path, subcommand, matches),
        _ => unreachable!()
    }
}
