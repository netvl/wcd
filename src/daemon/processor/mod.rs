use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, Utc};

use common::proto;
use common::config::ValidatedServerConfig;
use daemon::scheduler::Scheduler;
use daemon::processor::playlist::Playlist;
use daemon::stats::Stats;

mod playlist;
mod command;

pub const TRIGGER_JOB_NAME: &'static str = "trigger";
pub const REFRESH_JOB_NAME: &'static str = "refresh";
pub const UPDATE_STATS_JOB_NAME: &'static str = "update_stats";

pub const UPDATE_STATS_INTERVAL_SECS: i64 = 5;
pub const SKIP_INTERVAL_SECS: i64 = 10;

pub type ProcessorResult<T> = Result<T, ()>;  // unit for now

struct State {
    playlists: Vec<Playlist>,
    playlist_indices: HashMap<String, usize>,
    current_playlist: usize,
    scheduler: Scheduler,
    stats: Option<Stats>,
    last_trigger_time: Option<DateTime<Utc>>,
}

impl State {
    fn trigger(&mut self, simulate: bool) -> ProcessorResult<()> {
        {
            let playlist = &mut self.playlists[self.current_playlist];
            while {
                playlist.move_to_next_image();
                !playlist.apply_current_image()
            } { }
        }

        let _ = self.with_stats_and_current_path::<_, Box<Error>>(|stats, current| {
            stats.register_displays(current, 1)?;

            if let Some(last_trigger_time) = self.last_trigger_time {
                let now = Utc::now();

                let diff = now.signed_duration_since(last_trigger_time);
                if diff.num_seconds() <= SKIP_INTERVAL_SECS {
                    stats.register_skips(current, 1)?;
                }
            }

            Ok(())
        });

        self.last_trigger_time = Some(Utc::now());

        if simulate {
            self.scheduler.simulate(TRIGGER_JOB_NAME);
        }

        Ok(())
    }

    fn refresh_playlists(&mut self, simulate: bool) -> ProcessorResult<()> {
        for playlist in &mut self.playlists {
            playlist.update();
        }

        if simulate {
            self.scheduler.simulate(REFRESH_JOB_NAME);
        }

        Ok(())
    }

    fn playlists_by_name<'a>(&'a self) -> Box<Iterator<Item=(&'a str, &'a Playlist)> + 'a> {
        Box::new(
            self.playlist_indices.iter().map(move |(name, &idx)| (&**name, &self.playlists[idx]))
        )
    }

    fn get_status(&self) -> ProcessorResult<proto::StatusInfo> {
        let last_timestamp = match self.scheduler.get_last_execution_timestamp(TRIGGER_JOB_NAME) {
            Some(timestamp) => timestamp,
            None => return Err(())
        };

        let mut result = HashMap::new();
        for (name, playlist) in self.playlists_by_name() {
            let files = playlist.len();
            let mode = playlist.config().mode.into();
            let current_image = playlist.current().map(|img| img.path.display().to_string());
            let change_every = playlist.config().change_every;

            result.insert(name.to_owned(), proto::PlaylistInfo {
                directories: playlist.config().directories.iter()
                    .map(|pb| pb.to_string_lossy().into_owned())
                    .collect(),
                files: playlist.config().files.iter()
                    .map(|pb| pb.to_string_lossy().into_owned())
                    .collect(),
                total_files: files as u64,
                mode: mode,
                current_image: current_image,
                next_update: (last_timestamp + change_every).timestamp(),
                use_last_on_select: playlist.config().use_last_on_select,
                trigger_on_select: playlist.config().trigger_on_select
            });
        }

        let current_playlist = self.playlist_indices.iter()
            .find(|&(_, idx)| *idx == self.current_playlist)
            .unwrap().0.clone();

        Ok(proto::StatusInfo {
            playlists: result,
            current_playlist: current_playlist,
            last_update: last_timestamp.timestamp(),
        })
    }

    fn change_playlist(&mut self, playlist_name: &str) -> ProcessorResult<()> {
        if let Some(&idx) = self.playlist_indices.get(playlist_name) {
            info!("Changing the current playlist to {}", playlist_name);
            self.current_playlist = idx;

            macro_rules! current_playlist {
                () => { &mut self.playlists[self.current_playlist] }
            }
            self.scheduler.schedule(TRIGGER_JOB_NAME, current_playlist!().config().change_every);

            if current_playlist!().config().trigger_on_select {
                if current_playlist!().config().use_last_on_select {
                    if current_playlist!().move_to_next_image_if_first_time() {
                        info!("Playlist wasn't used before, picking the first image");
                        let _ = self.with_stats_and_current_path(|stats, current| {
                            stats.register_displays(current, 1)
                        });
                        
                    } else {
                        info!("Restoring the last used wallpaper in the current playlist");
                    }
                    current_playlist!().apply_current_image();
                    self.scheduler.simulate(TRIGGER_JOB_NAME);
                } else {
                    info!("Picking the next image from the playlist");
                    return self.trigger(false);
                }
            }

            Ok(())
        } else {
            Err(())
        }
    }

    fn update_stats(&self) -> ProcessorResult<()> {
        self.with_stats_and_current_path(|stats, current| {
            stats.register_display_time(current, UPDATE_STATS_INTERVAL_SECS)
        })
    }

    fn with_stats_and_current_path<F, E>(&self, f: F) -> ProcessorResult<()>
        where F: Fn(&Stats, &str) -> Result<(), E>,
              E: ::std::fmt::Display,
    {
        if let Some(ref stats) = self.stats {
            if let Some(current) = self.playlists[self.current_playlist].current() {
                f(stats, &current.path.to_string_lossy())
                    .map_err(|e| { warn!("Failed to update statistics: {}", e); })?;
            }
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Processor {
    state: Arc<Mutex<State>>,
}

impl Processor {
    pub fn new(config: ValidatedServerConfig,
               scheduler: Scheduler,
               stats: Option<Stats>) -> Processor {
        // will contain runtime playlists
        let mut playlists = Vec::new();
        // playlist name -> playlist index in the above vector
        let mut playlist_indices = HashMap::new();

        // build runtime playlist representation and collect some statistics
        let mut scanned_files = 0;
        let mut scanned_directories = 0;
        let mut scanned_playlists = 0;
        for (name, playlist) in config.playlists {
            scanned_files += playlist.files.len();
            scanned_directories += playlist.directories.len();
            scanned_playlists += 1;

            playlist_indices.insert(name, playlists.len());
            playlists.push(Playlist::from_config(playlist));
        }

        // obtain final bits of statistics and print them
        let total_files: usize = playlists.iter().map(|pl| pl.len()).sum();
        info!(
            "Scanned {} files and {} directories across {} playlists, found {} valid image files",
            scanned_files, scanned_directories, scanned_playlists, total_files
        );

        // set the default playlist
        // always unwraps
        let current_playlist = playlist_indices[&config.default_playlist];
        info!("Current playlist is {}", config.default_playlist);

        let state = State {
            playlists, playlist_indices, current_playlist, scheduler, stats,
            last_trigger_time: None,
        };
        let state = Arc::new(Mutex::new(state));
        Processor { state, }
    }

    pub fn start(&self) {
        let g = self.state.lock().unwrap();
        g.scheduler.schedule(TRIGGER_JOB_NAME, g.playlists[g.current_playlist].config().change_every);
    }

    pub fn trigger(&self, simulate: bool) -> ProcessorResult<()> {
        let mut g = self.state.lock().unwrap();
        g.trigger(simulate)
    }

    pub fn refresh_playlists(&self, simulate: bool) -> ProcessorResult<()> {
        let mut g = self.state.lock().unwrap();
        g.refresh_playlists(simulate)
    }

    pub fn get_status(&self) -> ProcessorResult<proto::StatusInfo> {
        let g = self.state.lock().unwrap();
        g.get_status()
    }

    pub fn change_playlist(&self, playlist_name: &str) -> ProcessorResult<()> {
        let mut g = self.state.lock().unwrap();
        g.change_playlist(playlist_name)
    }

    pub fn update_stats(&self) -> ProcessorResult<()> {
        let g = self.state.lock().unwrap();
        g.update_stats()
    }
}

