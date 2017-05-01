#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate chan;
#[macro_use(crate_version)] extern crate clap;
extern crate chrono;
extern crate toml;
extern crate serde;
extern crate bincode;
extern crate shellexpand;
extern crate log4rs;
extern crate appdirs;
extern crate nanomsg;
extern crate rand;

use std::borrow::Cow;
use std::path::Path;

use clap::{App, SubCommand, AppSettings, Arg};

mod common;
mod cli;
mod daemon;

fn main() {
    let matches = App::new("wcd")
        .version(crate_version!())
        .author("Vladimir Matveev <vladimir.matweev@gmail.com>")
        .about("A wallpaper change daemon and its control utility.")
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            Arg::from_usage("-c, --config=[FILE] 'Path to the configuration file'")
                .default_value("~/.config/wcd/config.toml")
        )
        .subcommand(daemon::subcommand())
        .subcommands(cli::subcommands())
        .get_matches();

    let config_path = matches.value_of("config").unwrap();
    let config_path: Cow<Path> = common::util::str_to_path(config_path);

    match matches.subcommand() {
        ("daemon", Some(matches)) => daemon::main(config_path, matches),
        (subcommand, Some(matches)) => cli::main(config_path, subcommand, matches),
        _ => unreachable!()
    }
}
