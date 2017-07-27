use std::collections::HashMap;
use std::error::Error;

use chrono::{DateTime, Utc};

use common::proto;
use common::config::ValidatedServerConfig;
use daemon::processor::playlist::Playlist;
use daemon::stats::Stats;

mod playlist;
mod command;

pub const TRIGGER_JOB_NAME: &'static str = "trigger";
pub const REFRESH_JOB_NAME: &'static str = "refresh";
pub const UPDATE_STATS_JOB_NAME: &'static str = "update_stats";

pub const UPDATE_STATS_INTERVAL_SECS: i64 = 5;
pub const SKIP_INTERVAL_SECS: i64 = 10;

pub type ProcessorResult<T> = Result<T, Box<Error>>;

pub struct State {
    playlists: Vec<Playlist>,
    playlist_indices: HashMap<String, usize>,
    current_playlist: usize,
    last_trigger_time: Option<DateTime<Utc>>,
    last_used_image: Option<String>,
}

impl State {
    pub fn new(config: ValidatedServerConfig) -> State {
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

        State {
            playlists, playlist_indices, current_playlist,
            last_trigger_time: None, last_used_image: None,
        }
    }
}

#[derive(Clone)]
pub struct Processor {
    daemon: super::Daemon,
}

impl Processor {
    pub fn new(daemon: super::Daemon) -> Processor {
        Processor { daemon, }
    }

    fn with_actions<F, T>(&self, f: F) -> T where F: FnOnce(StateActions) -> T {
        let state = self.daemon.state.lock();
        let mut processor_state = state.processor.borrow_mut();
        f(StateActions {
            state: &mut *processor_state,
            daemon: self.daemon.clone(),
        })
    }

    pub fn start(&self) {
        self.with_actions(|a| a.start());
    }

    pub fn trigger(&self, simulate: bool) -> ProcessorResult<()> {
        self.with_actions(|mut a| a.trigger(simulate))
    }

    pub fn refresh_playlists(&self, simulate: bool) -> ProcessorResult<()> {
        self.with_actions(|mut a| a.refresh_playlists(simulate))
    }

    pub fn get_status(&self) -> ProcessorResult<proto::StatusInfo> {
        self.with_actions(|a| a.get_status())
    }

    pub fn change_playlist(&self, playlist_name: &str) -> ProcessorResult<()> {
        self.with_actions(|mut a| a.change_playlist(playlist_name))
    }

    pub fn update_stats(&self) -> ProcessorResult<()> {
        self.with_actions(|a| a.update_stats())
    }
}

struct StateActions<'a> {
    state: &'a mut State,
    daemon: super::Daemon,
}

impl<'a> StateActions<'a> {
    fn start(&self) {
        self.daemon.scheduler().schedule(
            TRIGGER_JOB_NAME,
            self.state.playlists[self.state.current_playlist].config().change_every
        );
    }
    
    fn trigger(&mut self, simulate: bool) -> ProcessorResult<()> {
        self.state.last_used_image = self.state.playlists[self.state.current_playlist].current()
            .map(|image| image.path.to_string_lossy().into_owned());

        {
            let playlist = &mut self.state.playlists[self.state.current_playlist];
            while {
                playlist.move_to_next_image();
                !playlist.apply_current_image()
            } { }
        }

        self.with_stats_and_current_path::<_, Box<Error>>(|stats, current| {
            stats.register_displays(current, 1)?;

            if let Some(last_trigger_time) = self.state.last_trigger_time {
                let now = Utc::now();

                let diff = now.signed_duration_since(last_trigger_time);
                if diff.num_seconds() <= SKIP_INTERVAL_SECS {
                    let last_used_image = self.state.last_used_image.as_ref().unwrap();
                    stats.register_skips(last_used_image, 1)?;
                }
            }

            Ok(())
        });

        self.state.last_trigger_time = Some(Utc::now());

        if simulate {
            self.daemon.scheduler().simulate(TRIGGER_JOB_NAME);
        }

        Ok(())
    }

    fn refresh_playlists(&mut self, simulate: bool) -> ProcessorResult<()> {
        for playlist in &mut self.state.playlists {
            playlist.update();
        }

        if simulate {
            self.daemon.scheduler().simulate(REFRESH_JOB_NAME);
        }

        Ok(())
    }

    fn get_status(&self) -> ProcessorResult<proto::StatusInfo> {
        simple_error!(MissingTimestamp, "No last execution timestamp of image change trigger is available");

        let last_timestamp = match self.daemon.scheduler().get_last_execution_timestamp(TRIGGER_JOB_NAME) {
            Some(timestamp) => timestamp,
            None => return Err(MissingTimestamp.into())
        };

        fn playlists_by_name<'a>(state: &'a State) -> Box<Iterator<Item=(&'a str, &'a Playlist)> + 'a> {
            Box::new(
                state.playlist_indices.iter()
                    .map(move |(name, &idx)| (&**name, &state.playlists[idx]))
            )
        }

        let mut result = HashMap::new();
        for (name, playlist) in playlists_by_name(self.state) {
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

        let current_playlist = self.state.playlist_indices.iter()
            .find(|&(_, idx)| *idx == self.state.current_playlist)
            .unwrap().0.clone();

        Ok(proto::StatusInfo {
            playlists: result,
            current_playlist: current_playlist,
            last_update: last_timestamp.timestamp(),
        })
    }

    fn change_playlist(&mut self, playlist_name: &str) -> ProcessorResult<()> {
        if let Some(&idx) = self.state.playlist_indices.get(playlist_name) {
            info!("Changing the current playlist to {}", playlist_name);
            self.state.current_playlist = idx;

            macro_rules! current_playlist {
                () => { &mut self.state.playlists[self.state.current_playlist] }
            }
            self.daemon.scheduler().schedule(TRIGGER_JOB_NAME, current_playlist!().config().change_every);

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
                    self.daemon.scheduler().simulate(TRIGGER_JOB_NAME);
                } else {
                    info!("Picking the next image from the playlist");
                    return self.trigger(false);
                }
            }

            Ok(())
        } else {
            simple_error!(PlaylistDoesNotExist, "Specified playlist does not exist");
            Err(PlaylistDoesNotExist.into())
        }
    }

    pub fn update_stats(&self) -> ProcessorResult<()> {
        self.with_stats_and_current_path(|stats, current| {
            stats.register_display_time(current, UPDATE_STATS_INTERVAL_SECS)
        });
        Ok(())
    }

    fn with_stats_and_current_path<F, E>(&self, f: F)
        where F: Fn(&Stats, &str) -> Result<(), E>,
              E: ::std::fmt::Display,
    {
        if let Some(ref stats) = self.daemon.stats() {
            if let Some(current) = self.state.playlists[self.state.current_playlist].current() {
                if let Err(e) = f(stats, &current.path.to_string_lossy()) {
                    warn!("Failed to update statistics: {}", e);
                }
            }
        }
    }
}
