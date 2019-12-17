#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_migrations;
#[cfg(feature = "stats-analyzer")]
extern crate gtk;
#[cfg(feature = "stats-analyzer")]
extern crate gdk;
#[cfg(feature = "stats-analyzer")]
extern crate gdk_pixbuf;
#[cfg(feature = "stats-analyzer")]
extern crate cairo;

use std::path::PathBuf;

use structopt::StructOpt;

use common::log::LogLevel;

#[macro_use]
mod macros;
mod common;
mod cli;
mod daemon;
#[cfg(feature = "stats-analyzer")]
mod stats_analyzer;

#[derive(Debug, StructOpt)]
#[structopt(name = "wcd", about = "A wallpaper change daemon and its control utility.")]
struct Options {
    #[structopt(short, long, default_value = "~/.config/wcd/config.toml")]
    config: PathBuf,

    #[structopt(short, long, parse(from_occurrences))]
    verbose: u32,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
enum Command {
    /// Starts the wallpaper change daemon.
    Start,
    /// Triggers the wallpaper change in the current playlist.
    Trigger {
        /// Refresh the currently selected wallpaper.
        #[structopt(short, long)]
        keep: bool,
    },
    /// Makes wcd rescan all directories in all playlists, potentially loading new files.
    Refresh,
    /// Shuts wcd down.
    Terminate,
    /// Displays the current status information (available playlists, current items in them, timestamps, etc).
    Status,
    /// Sets the given playlist as the current one (may cause immediate wallpaper switch, depending on the
    /// selected playlist configuration).
    SetPlaylist {
        name: String,

        #[structopt(long)]
        or_trigger: bool,
    },
    #[cfg(target_feature = "stats-analyzer")]
    StatsAnalyzer,
}

fn main() {
    let options: Options = Options::from_args();

    #[cfg(feature = "stats-analyzer")]
    let app = app
        .subcommand(stats_analyzer::subcommand());

    let log_level = match options.verbose {
        0 => LogLevel::Normal,
        1 => LogLevel::Debug,
        _ => LogLevel::Trace,
    };
    common::log::configure_or_panic(log_level);

    match options.cmd {
        Command::Start => daemon::main(&options.config),
        #[cfg(target_feature = "stats-analyzer")]
        Command::StatsAnalyzer => stats_analyzer::main(&options.config),
        subcommand => cli::main(&options.config, subcommand),
    }
}
