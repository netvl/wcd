use std::borrow::Cow;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::cell::RefCell;

use clap::App;
use chrono::Duration;
use parking_lot::ReentrantMutex;

use common::config;

mod control;
mod scheduler;
mod processor;
mod stats;

pub const SUBCOMMAND_NAME: &'static str = "daemon";

pub fn subcommand() -> App<'static, 'static> {
    App::new(SUBCOMMAND_NAME)
        .about("Starts the wallpaper change daemon")
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

    let stats_state = match config.stats_db.as_ref() {
        Some(p) => match stats::State::new(p) {
            Ok(state) => {
                info!("Collecting statistics is enabled, statistics database is {}", p.display());
                Some(state)
            },
            Err(e) => {
                warn!("Failed to initialize the statistics database in {}: {}", p.display(), e);
                None
            }
        },
        None => {
            info!("Collecting statistics is disabled");
            None
        },
    };
    let scheduler_state = scheduler::State::new();
    let processor_state = processor::State::new(config);

    let daemon = Daemon {
        state: Arc::new(ReentrantMutex::new(State {
            processor: RefCell::new(processor_state),
            scheduler: RefCell::new(scheduler_state),
            stats: stats_state.map(RefCell::new),
        }))
    };

    let control = control::Control::new(endpoint, daemon.clone());

    {
        let d = daemon.clone();
        daemon.scheduler().prepare(processor::TRIGGER_JOB_NAME, move || {
            let daemon = d.clone();
            move || {
                if let Err(_) = daemon.processor().trigger(false) {
                    warn!("Failed to run scheduled wallpaper change");
                }
            }
        });
    }

    if let config::WatchMode::Poll(watch_duration) = watch_mode {
        let d = daemon.clone();
        daemon.scheduler().prepare(processor::REFRESH_JOB_NAME, move || {
            let daemon = d.clone();
            move || {
                if let Err(_) = daemon.processor().refresh_playlists(false) {
                    warn!("Failed to refresh playlists");
                }
            }
        });
        daemon.scheduler().schedule(processor::REFRESH_JOB_NAME, watch_duration);
    }

    if daemon.stats().is_some() {
        let d = daemon.clone();
        daemon.scheduler().prepare(processor::UPDATE_STATS_JOB_NAME, move || {
            let daemon = d.clone();
            move || {
                if let Err(_) = daemon.processor().update_stats() {
                    warn!("Failed to update statistics");
                }
            }
        });
        daemon.scheduler().schedule(
            processor::UPDATE_STATS_JOB_NAME,
            Duration::seconds(processor::UPDATE_STATS_INTERVAL_SECS)
        );
    }

    daemon.processor().start();
    let scheduler_handle = daemon.scheduler().start();
    let control_handle = control.start();

    let _ = control_handle.join();
    let _ = scheduler_handle.join();
}

struct State {
    processor: RefCell<processor::State>,
    scheduler: RefCell<scheduler::State>,
    stats: Option<RefCell<stats::State>>,
}

#[derive(Clone)]
pub struct Daemon {
    state: Arc<ReentrantMutex<State>>,
}

impl Daemon {
    fn scheduler(&self) -> scheduler::Scheduler {
        scheduler::Scheduler::new(self.clone())
    }

    fn processor(&self) -> processor::Processor {
        processor::Processor::new(self.clone())
    }

    fn stats(&self) -> Option<stats::Stats> {
        self.state.lock().stats.as_ref()
            .map(|_| stats::Stats::new(self.clone()))
    }
}
