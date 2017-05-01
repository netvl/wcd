use log4rs;
use log4rs::config::*;
use log4rs::append::console::ConsoleAppender;
use log4rs::encode::pattern::PatternEncoder;
use log::LogLevelFilter;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum LogLevel {
    Normal,
    Debug,
    Trace,
}

fn build_log_config(level: LogLevel) -> Config {
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

pub fn configure_or_panic(level: LogLevel) {
    let log_config = build_log_config(level);
    log4rs::init_config(log_config).unwrap();
}
