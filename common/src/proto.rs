use std::collections::HashMap;

use config;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq)]
pub enum ControlRequest {
    Ping,
    TriggerChange,
    ForceUpdate,
    Terminate,

    GetStatus,

    ChangePlaylist(String),
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq)]
pub enum ControlResponse {
    PingOk,
    TriggerChangeOk,
    ForceUpdateOk,
    TerminateOk,

    StatusInfo {
        playlists: HashMap<String, PlaylistInfo>,
        current_playlist: String,
    },

    ChangePlaylistOk,
    ChangePlaylistFailed,
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq)]
pub struct ControlEnvelope<T> {
    pub version: String,
    pub content: T
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq)]
pub struct PlaylistInfo {
    pub files: u64,
    pub mode: ChangeMode,
    pub current_image: Option<String>
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
