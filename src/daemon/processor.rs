use std::fmt::Write;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::ffi::OsStr;
use std::process::Command;
use std::io;
use std::fs;

use rand::{self, Rng};
use lazy_scoped::Lazy;

use common::proto;
use common::config::{ValidatedServerConfig, ValidatedPlaylist, ChangeMode};
use daemon::scheduler::Scheduler;

pub const TRIGGER_JOB_NAME: &'static str = "trigger";
pub const REFRESH_JOB_NAME: &'static str = "refresh";

pub type ProcessorResult<T> = Result<T, ()>;  // unit for now

struct Image {
    // path to the image file
    path: PathBuf,
    // whether or not this image is usable, i.e. if the corresponding file is accessible
    usable: bool,
}

struct Playlist {
    // a sequence of all currently known images in the current playlist
    images: Vec<Image>,
    // a sequence of indices into the above array of images which weren't used yet
    unused: Vec<usize>,
    // an index into images array of the currently used image in this playlist
    current: Option<usize>,
    // configuration
    config: ValidatedPlaylist,
}

impl Playlist {
    fn from_config(config: ValidatedPlaylist) -> Playlist {
        let images = build_images_list(&config);
        let mut unused: Vec<_> = (0..images.len()).collect();
        if config.mode == ChangeMode::Random {
            rand::thread_rng().shuffle(&mut unused);
        }
        Playlist {
            images: images,
            unused: unused,
            config: config,
            current: None
        }
    }

    fn shuffle_unused_if_needed(&mut self) {
        if self.config.mode == ChangeMode::Random {
            rand::thread_rng().shuffle(&mut self.unused);
        }
    }

    fn update(&mut self) {
        let new_images = update_images_list(&self.config, &self.images);
        let new_indices = (self.images.len()..).take(new_images.len());
        if !new_images.is_empty() {
            self.images.extend(new_images);
            self.unused.extend(new_indices);
            self.shuffle_unused_if_needed();
        }
    }

    fn move_to_next_image_if_first_time(&mut self) -> bool {
        if self.current.is_none() {
            self.move_to_next_image();
            true
        } else {
            false
        }
    }

    fn move_to_next_image(&mut self) {
        match self.unused.pop() {
            Some(idx) => self.current = Some(idx),
            None => {
                let was_nonempty = !self.images.is_empty();

                // leave only usable images
                self.images.retain(|img| img.usable);

                if self.images.is_empty() {
                    if was_nonempty {
                        warn!("Current playlist became empty, no further updates will be done");
                    }
                    self.current = None;
                } else {
                    info!("Current playlist has ended, restarting");
                    self.unused.extend(0..self.images.len());
                    self.shuffle_unused_if_needed();
                    self.move_to_next_image();
                }

            }
        }
    }

    fn apply_current_image(&mut self) -> bool {
        if let Some(current) = self.current {
            let command = ChangeCommand::from_config(&self.config);
            info!("Changing wallpaper to {}", self.images[current].path.display());
            command.execute(&mut self.images[current])
        } else {
            info!("No wallpapers are available in the current playlist");
            true
        }
    }
}

struct State {
    playlists: Vec<Playlist>,
    playlist_indices: HashMap<String, usize>,
    current_playlist: usize,
    scheduler: Scheduler,
}

impl State {
    fn trigger(&mut self, simulate: bool) -> ProcessorResult<()> {
        let playlist = &mut self.playlists[self.current_playlist];
        while {
            playlist.move_to_next_image();
            !playlist.apply_current_image()
        } { }

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
            let files = playlist.images.len();
            let mode = playlist.config.mode.into();
            let current_image = playlist.current
                .map(|idx| playlist.images[idx].path.display().to_string());
            let change_every = playlist.config.change_every;

            result.insert(name.to_owned(), proto::PlaylistInfo {
                directories: playlist.config.directories.iter()
                    .map(|pb| pb.to_string_lossy().into_owned())
                    .collect(),
                files: playlist.config.files.iter()
                    .map(|pb| pb.to_string_lossy().into_owned())
                    .collect(),
                total_files: files as u64,
                mode: mode,
                current_image: current_image,
                next_update: (last_timestamp + change_every).timestamp(),
                use_last_on_select: playlist.config.use_last_on_select,
                trigger_on_select: playlist.config.trigger_on_select
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
            self.scheduler.schedule(TRIGGER_JOB_NAME, current_playlist!().config.change_every);

            if current_playlist!().config.trigger_on_select {
                if current_playlist!().config.use_last_on_select {
                    if current_playlist!().move_to_next_image_if_first_time() {
                        info!("Playlist wasn't used before, picking the first image");
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
}

#[derive(Clone)]
pub struct Processor {
    state: Arc<Mutex<State>>,
}

impl Processor {
    pub fn new(config: ValidatedServerConfig,
           scheduler: Scheduler) -> Processor {
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
        let total_files: usize = playlists.iter().map(|pl| pl.images.len()).sum();
        info!(
            "Scanned {} files and {} directories across {} playlists, found {} valid image files",
            scanned_files, scanned_directories, scanned_playlists, total_files
        );

        // set the default playlist
        // always unwraps
        let current_playlist = playlist_indices[&config.default_playlist];
        info!("Current playlist is {}", config.default_playlist);

        let state = State {
            playlists: playlists,
            playlist_indices: playlist_indices,
            current_playlist: current_playlist,
            scheduler: scheduler,
        };
        Processor {
            state: Arc::new(Mutex::new(state)),
        }
    }

    pub fn start(&self) {
        let g = self.state.lock().unwrap();
        g.scheduler.schedule(TRIGGER_JOB_NAME, g.playlists[g.current_playlist].config.change_every);
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
}

fn build_images_list(config: &ValidatedPlaylist) -> Vec<Image> {
    let mut files: Vec<_> = Vec::new();

    for file in &config.files {
        if check_file(file) {
            files.push(Image {
                path: file.clone(),
                usable: true
            });
        }
    }

    for dir in &config.directories {
        if let Err(e) = scan_directory(dir, &mut files) {
            warn!("Error reading directory {}: {}", dir.display(), e);
        }
    }

    files
}

fn update_images_list(config: &ValidatedPlaylist, existing_images: &[Image]) -> Vec<Image> {
    let mut result = Vec::new();
    for dir in &config.directories {
        match rescan_directory(dir, existing_images, &mut result) {
            Ok(n) if n > 0 => info!("Discovered {} more files in directory {}", n, dir.display()),
            Ok(_) => {}
            Err(e) => warn!("Error reading directory {}: {}", dir.display(), e)
        }
    }
    result
}

fn rescan_directory(dir: &Path, existing_images: &[Image],
                    result: &mut Vec<Image>) -> io::Result<usize>
{
    let mut n = 0;

    for entry in fs::read_dir(dir)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Error reading directory entry inside {}: {}", dir.display(), e);
                continue;
            }
        };

        let path = entry.path();
        if !existing_images.iter().map(|i| &i.path).any(|p| p == &path) && check_file(&path) {
            n += 1;
            result.push(Image {
                path: path,
                usable: true
            });
        }
    }

    Ok(n)
}

fn scan_directory(dir: &Path, images: &mut Vec<Image>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                warn!("Error reading directory entry inside {}: {}", dir.display(), e);
                continue;
            }
        };

        let path = entry.path();
        if check_file(&path) {
            images.push(Image {
                path: path,
                usable: true
            });
        }
    }
    Ok(())
}

fn check_file(file: &Path) -> bool {
    if check_file_format(file) {
        match check_file_access(file) {
            Ok(r) => r,
            Err(e) => {
                warn!("Error accessing file {}: {}", file.display(), e);
                false
            }
        }
    } else {
        false
    }
}

fn check_file_format(file: &Path) -> bool {
    if let Some(ext) = file.extension() {
        if let Some(ext) = ext.to_str() {
            match &*ext.to_lowercase() {
                "jpg" | "jpeg" | "png" | "gif" => return true,
                _ => {}
            }
        }
    }
    false
}

fn check_file_access(file: &Path) -> io::Result<bool> {
    fs::metadata(file).map(|m| m.is_file())
}

struct ChangeCommand<'a> {
    name: &'a str,
    args: &'a [String]
}

impl<'a> ChangeCommand<'a> {
    fn from_config(config: &'a ValidatedPlaylist) -> ChangeCommand<'a> {
        ChangeCommand {
            name: &config.command,
            args: &config.command_args
        }
    }

    fn execute(&self, image: &mut Image) -> bool {
        if !check_file(&image.path) {
            warn!("Marking image {} as unusable", image.path.display());
            image.usable = false;
            return false;
        }

        let args: Vec<&OsStr> = self.args.iter()
            .map(|a| if a == "{}" { image.path.as_os_str() } else { a.as_ref() })
            .collect();

        let command_str = Lazy::new(|| {
            let mut result_str = if self.name.contains(char::is_whitespace) {
                format!("\"{}\"", self.name)
            } else {
                self.name.to_owned()
            };

            for a in &args {
                let s = a.to_string_lossy();
                let _ = if s.contains(char::is_whitespace) {
                    write!(&mut result_str, " \"{}\"", a.to_string_lossy())
                } else {
                    write!(&mut result_str, " {}", a.to_string_lossy())
                };
            }

            result_str
        });

        debug!("Executing command: {}", *command_str);

        let status = Command::new(&self.name).args(&args).status();
        match status {
            Ok(ref status) if status.success() => {},
            status => {
                match status {
                    Ok(status) => match status.code() {
                        Some(code) =>
                            warn!("Command '{}' has exited with code {}", *command_str, code),
                        None =>
                            warn!("Command '{}' has exited without status code", *command_str),
                    },
                    Err(e) => warn!("Failed to start command '{}': {}", *command_str, e),
                }
            }
        }

        true
    }
}
