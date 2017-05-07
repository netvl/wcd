use std::path::PathBuf;

use rand::{self, Rng};

use common::config::{ValidatedPlaylist, ChangeMode};
use daemon::processor::command::ChangeCommand;

mod files;

#[derive(Clone, Eq, PartialEq)]
pub struct Image {
    // path to the image file
    pub path: PathBuf,
    // whether or not this image is usable, i.e. if the corresponding file is accessible
    pub usable: bool,
}

impl Image {
    pub fn check(&self) -> bool {
        files::check_file(&self.path)
    }
}

pub struct Playlist {
    // a sequence of all currently known images in the current playlist
    images: Vec<Image>,
    // a sequence of of images which weren't used yet
    unused: Vec<Image>,
    // the currently used image in this playlist
    current: Option<Image>,
    // configuration
    config: ValidatedPlaylist,
}

impl Playlist {
    pub fn from_config(config: ValidatedPlaylist) -> Playlist {
        let images = files::scan_playlist(&config);
        let mut unused: Vec<_> = images.clone();
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

    pub fn config(&self) -> &ValidatedPlaylist {
        &self.config
    }

    pub fn len(&self) -> usize { self.images.len() }

    pub fn current(&self) -> Option<&Image> { self.current.as_ref() }

    fn shuffle_unused_if_needed(&mut self) {
        if self.config.mode == ChangeMode::Random {
            rand::thread_rng().shuffle(&mut self.unused);
        }
    }

    pub fn update(&mut self) {
        let files::RescanResult { added, removed } = files::rescan_playlist(&self.config, &self.images);
        let added_new_items = !added.is_empty();

        let removed_pred = |img: &Image| !removed.iter().any(|removed_img| removed_img.path == img.path);
        self.images.retain(&removed_pred);
        self.unused.retain(&removed_pred);

        self.images.extend(added.iter().cloned());
        self.unused.extend(added);

        if added_new_items {
            self.shuffle_unused_if_needed();
        }
    }

    pub fn move_to_next_image_if_first_time(&mut self) -> bool {
        if self.current.is_none() {
            self.move_to_next_image();
            true
        } else {
            false
        }
    }

    pub fn move_to_next_image(&mut self) {
        match self.unused.pop() {
            Some(image) => self.current = Some(image),
            None => {
                let was_nonempty = !self.images.is_empty();

                // leave only usable images
                // TODO: maybe remove?
                self.images.retain(|img| img.usable);

                if self.images.is_empty() {
                    if was_nonempty {
                        warn!("Current playlist became empty, no further updates will be done");
                    }
                    self.current = None;
                } else {
                    info!("Current playlist has ended, restarting");
                    self.unused.extend(self.images.iter().cloned());
                    self.shuffle_unused_if_needed();
                    self.move_to_next_image();
                }

            }
        }
    }

    pub fn apply_current_image(&mut self) -> bool {
        if let Some(ref current) = self.current {
            let command = ChangeCommand::from_config(&self.config);
            info!("Changing wallpaper to {}", current.path.display());
            if !command.execute(current) {
                warn!("Failed to execute the change ");
                false
            } else {
                true
            }
        } else {
            info!("No wallpapers are available in the current playlist");
            true
        }
    }
}

