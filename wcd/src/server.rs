use std::path::{PathBuf, Path};
use std::io::{self, Write};
use std::process::Command;
use std::fs;
use std::fmt::Write as FmtWrite;
use std::thread::{self, JoinHandle};
use std::time::Duration as StdDuration;
use std::collections::HashMap;
use std::ffi::OsStr;

use chan::{Receiver, Sender};
use rand::{self, Rng};
use chrono::{DateTime, UTC};

use wcd_common::proto::{self, ControlRequest, ControlResponse};
use wcd_common::config::{ValidatedServerConfig, ValidatedPlaylist, ChangeMode, WatchMode};

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
    
    fn shuffle_unused(&mut self) {
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
            self.shuffle_unused();
        }
    }
    
    fn move_to_next(&mut self) {
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
                    self.shuffle_unused();
                    self.move_to_next();
                }

            }
        }
    }

    fn apply_current(&mut self) -> bool {
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
    watch: WatchMode,
    last_timestamp: DateTime<UTC>,
    last_watch_timestamp: DateTime<UTC>,
    control_req_recv: Receiver<ControlRequest>,
    control_resp_send: Sender<ControlResponse>,
}

impl State {
    fn start_loop(mut self) {
        let (
            control_req_recv, control_resp_send,
            mut playlists, playlist_indices,
            mut current_playlist
        ) = (
            self.control_req_recv, self.control_resp_send,
            self.playlists, self.playlist_indices,
            self.current_playlist
        );
        loop {
            chan_select! {
                default => {
                    thread::sleep(StdDuration::from_secs(1));  // sleep 1 second
                },
                control_req_recv.recv() -> value => {
                    if let Some(value) = value {
                        match value {
                            ControlRequest::Ping => control_resp_send.send(ControlResponse::PingOk),
                            ControlRequest::TriggerChange => {
                                self.last_timestamp = past_timestamp();
                                control_resp_send.send(ControlResponse::TriggerChangeOk);
                            }
                            ControlRequest::ForceUpdate => {
                                self.last_watch_timestamp = past_timestamp();
                                control_resp_send.send(ControlResponse::ForceUpdateOk);
                            }
                            ControlRequest::Terminate => {
                                info!("Received termination request from client, exiting");
                                control_resp_send.send(ControlResponse::TerminateOk);
                                break;
                            }
                            ControlRequest::GetStatus => {
                                let mut result = HashMap::new();
                                for (name, playlist) in playlist_indices.iter()
                                    .map(|(name, idx)| (name, &playlists[*idx])) {
                                    let files = playlist.images.len();
                                    let mode = proto::ChangeMode::from_config(playlist.config.mode);
                                    let current_image = playlist.current.map(|idx| playlist.images[idx].path.display().to_string());
                                    result.insert(name.clone(), proto::PlaylistInfo {
                                        files: files as u64,
                                        mode: mode,
                                        current_image: current_image
                                    });
                                }
                                let current_playlist = playlist_indices.iter()
                                    .find(|&(_, idx)| *idx == current_playlist)
                                    .unwrap().0.clone();
                                control_resp_send.send(ControlResponse::StatusInfo {
                                    playlists: result,
                                    current_playlist: current_playlist
                                });
                            }
                            ControlRequest::ChangePlaylist(name) => {
                                if let Some(&idx) = playlist_indices.get(&name) {
                                    info!("Changing current playlist to {}", name);
                                    current_playlist = idx;
                                    if playlists[current_playlist].config.trigger_on_select {
                                        if playlists[current_playlist].config.use_last_on_select {
                                            info!("Restoring last wallpaper in current playlist");
                                            playlists[current_playlist].apply_current();
                                            self.last_timestamp = UTC::now();
                                        } else {
                                            self.last_timestamp = past_timestamp();
                                        }
                                    }
                                    control_resp_send.send(ControlResponse::ChangePlaylistOk);
                                } else {
                                    control_resp_send.send(ControlResponse::ChangePlaylistFailed);
                                }
                            }
                        }
                    } else {
                        info!("Control channel was broken, exiting");
                        break;
                    }
                },
            }

            // refresh playlists if needed
            if let WatchMode::Poll(watch_interval) = self.watch {
                let current_watch_timestamp = UTC::now();
                let difference = current_watch_timestamp - self.last_watch_timestamp;

                if difference >= watch_interval {
                    info!("Refreshing playlists");
                    for playlist in &mut playlists {
                        playlist.update();
                    }
                    self.last_watch_timestamp = current_watch_timestamp;
                }
            }

            // change wallpaper if time has come
            let current_timestamp = UTC::now();
            let difference = current_timestamp - self.last_timestamp;

            if difference >= playlists[current_playlist].config.change_every {
                while { 
                    playlists[current_playlist].move_to_next();
                    !playlists[current_playlist].apply_current()
                } { }
                self.last_timestamp = current_timestamp;
            }
        }
    }
}

pub fn start(config: ValidatedServerConfig, 
             control_req_recv: Receiver<ControlRequest>,
             control_resp_send: Sender<ControlResponse>) -> JoinHandle<()> {
    thread::spawn(move || {
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
        let total_files = playlists.iter().map(|pl| pl.images.len()).fold(0, |a, b| a + b);
        info!(
            "Scanned {} files and {} directories across {} playlists, found {} valid image files",
            scanned_files, scanned_directories, scanned_playlists, total_files
        );

        // set the default playlist
        // always unwraps
        let current_playlist = *playlist_indices.get(&config.default_playlist).unwrap();
        info!("Current playlist is {}", config.default_playlist);

        let state = State {
            playlists: playlists,
            playlist_indices: playlist_indices,
            current_playlist: current_playlist,
            watch: config.watch,
            last_timestamp: past_timestamp(),
            last_watch_timestamp: UTC::now(),
            control_req_recv: control_req_recv,
            control_resp_send: control_resp_send,
        };

        state.start_loop();

    })
}

// returns an instant most likely located in the past
fn past_timestamp() -> DateTime<UTC> {
    DateTime::parse_from_str("0+0000", "%s%z")
            .unwrap()
            .with_timezone(&UTC)
}

fn build_images_list(config: &ValidatedPlaylist) -> Vec<Image> {
    let mut files: Vec<_> = Vec::new();

    for file in &config.files {
        if check_file(&file) {
            files.push(Image {
                path: file.clone(),
                usable: true
            });
        }
    }
    
    for dir in &config.directories {
        if let Err(e) = scan_directory(&dir, &mut files) {
            warn!("Error reading directory {}: {}", dir.display(), e);
        }
    }

    files
}

fn update_images_list(config: &ValidatedPlaylist, existing_images: &[Image]) -> Vec<Image> {
    let mut result = Vec::new();
    for dir in &config.directories {
        match rescan_directory(&dir, existing_images, &mut result) {
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

    for entry in try!(fs::read_dir(dir)) {
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
    for entry in try!(fs::read_dir(dir)) {
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
        debug!("Executing command {} with arguments: {:?}", self.name, args);

        let status = Command::new(&self.name).args(&args).status();
        match status {
            Ok(ref status) if status.success() => {},
            status => {
                let mut args_str = String::new();
                for a in args.iter() {
                    let _ = write!(&mut args_str, " \"{}\"", a.to_string_lossy());
                }
                let command_str = format!("\"{}\"{}", self.name, args_str);

                match status {
                    Ok(status) => match status.code() {
                        Some(code) =>
                            warn!("command {} has exited with code {}", command_str, code),
                        None =>
                            warn!("command {} has exited without status code", command_str)
                    },
                    Err(e) => warn!("failed to start command {}: {}", command_str, e)
                }
            }
        }

        true
    }
}
