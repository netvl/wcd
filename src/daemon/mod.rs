use std::borrow::Cow;
use std::path::Path;
use std::process;

use clap::{App, AppSettings, Arg, SubCommand, ArgMatches};
use log4rs;

use common::{config, util};

mod server;
mod control;
mod cache;

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name("daemon")
        .about("Starts the wallpaper change daemon")
        .setting(AppSettings::ColoredHelp)
        .args_from_usage(
            "-v... 'Enable verbose output (up to two times)'"
        )
}

pub fn main(config_path: Cow<Path>, matches: &ArgMatches) {
    let log_level = match matches.occurrences_of("v") {
        0 => LogLevel::Normal,
        1 => LogLevel::Debug,
        _ => LogLevel::Trace,
    };
    let log_config = build_log_config(log_level);
    log4rs::init_config(log_config).unwrap();

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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum LogLevel {
    Normal,
    Debug,
    Trace,
}

fn build_log_config(level: LogLevel) -> log4rs::config::Config {
    use log4rs::config::*;
    use log4rs::append::console::ConsoleAppender;
    use log4rs::encode::pattern::PatternEncoder;
    use log::LogLevelFilter;

    let encoder = PatternEncoder::new(match level {
        LogLevel::Normal | LogLevel::Debug => "[{h({l:.1})}] {m}{n}",
        LogLevel::Trace => "[{h({l:.1})}] [{f}:{L}] {m}{n}"
    });

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(encoder))
        .build();

    let appender = Appender::builder()
        .build("console", Box::new(stdout));

    let root = Root::builder()
        .appender("console")
        .build(match level {
            LogLevel::Normal => LogLevelFilter::Info,
            LogLevel::Debug => LogLevelFilter::Debug,
            LogLevel::Trace => LogLevelFilter::Trace,
        });

    Config::builder().appender(appender).build(root).unwrap()
}
