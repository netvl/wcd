use std::borrow::Cow;
use std::path::Path;
use std::process;

use clap::{App, AppSettings};

use common::config;

mod control;
mod cache;
mod scheduler;
mod processor;

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
    let watch_mode = config.server.watch.clone();
    let config = config.server;

    let scheduler = scheduler::Scheduler::new();
    let processor = processor::Processor::new(config, scheduler.clone());
    let control = control::Control::new(endpoint, processor.clone(), scheduler.clone());

    {
        let processor = processor.clone();
        scheduler.prepare(processor::TRIGGER_JOB_NAME, move || {
            if let Err(_) = processor.trigger(false) {
                warn!("Failed to run scheduled wallpaper change");
            }
        });
    }

    if let config::WatchMode::Poll(watch_duration) = watch_mode {
        let processor = processor.clone();
        scheduler.prepare(processor::REFRESH_JOB_NAME, move || {
            if let Err(_) = processor.refresh_playlists(false) {
                warn!("Failed to refresh playlists");
            }
        });
        scheduler.schedule(processor::REFRESH_JOB_NAME, watch_duration);
    }

    processor.start();
    let scheduler_handle = scheduler.start();
    let control_handle = control.start();

    let _ = control_handle.join();
    let _ = scheduler_handle.join();
}
