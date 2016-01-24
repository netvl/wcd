use std::collections::HashMap;

use config;

pub const VERSION: u32 = 1;

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq)]
pub enum ControlRequest {
    TriggerChange,
    RefreshPlaylists,
    Terminate,

    GetStatus,

    ChangePlaylist(String),
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq)]
pub enum ControlResponse {
    TriggerChangeOk,
    RefreshPlaylistsOk,
    TerminateOk,

    StatusInfo {
        playlists: HashMap<String, PlaylistInfo>,
        current_playlist: String,
        last_update: i64
    },

    ChangePlaylistOk,
    ChangePlaylistFailed,
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq)]
pub struct ControlEnvelope<T> {
    pub version: u32,
    pub content: T
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq)]
pub struct PlaylistInfo {
    pub files: u64,
    pub mode: ChangeMode,
    pub current_image: Option<String>,
    pub trigger_on_select: bool,
    pub use_last_on_select: bool,
    pub next_update: i64,
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq)]
pub enum ChangeMode {
    Sequential,
    Random
}

impl ChangeMode {
    pub fn from_config(mode: config::ChangeMode) -> ChangeMode {
        match mode {
            config::ChangeMode::Sequential => ChangeMode::Sequential,
            config::ChangeMode::Random => ChangeMode::Random
        }
    }
}
