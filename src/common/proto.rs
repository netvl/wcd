use std::collections::HashMap;

use common::config;
use common::grpc::wcd;

#[derive(Debug, Clone, PartialEq)]
pub enum ControlRequest {
    TriggerChange,
    RefreshPlaylists,
    Terminate,

    GetStatus,

    ChangePlaylist(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlResponse {
    TriggerChangeOk,
    RefreshPlaylistsOk,
    TerminateOk,

    StatusInfoOk(StatusInfo),
    StatusInfoFailed(String),

    ChangePlaylistOk,
    ChangePlaylistFailed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct StatusInfo {
    pub playlists: HashMap<String, PlaylistInfo>,
    pub current_playlist: String,
    pub last_update: i64,
}

impl From<wcd::StatusInfo> for StatusInfo {
    fn from(proto: wcd::StatusInfo) -> StatusInfo {
        StatusInfo {
            playlists: proto.playlists.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect(),
            current_playlist: proto.current_playlist,
            last_update: proto.last_update,
        }
    }
}

impl Into<wcd::StatusInfo> for StatusInfo {
    fn into(self) -> wcd::StatusInfo {
        let mut proto = wcd::StatusInfo::new();
        proto.set_playlists(self.playlists.into_iter().map(|(k, v)| (k, v.into())).collect());
        proto.set_current_playlist(self.current_playlist);
        proto.set_last_update(self.last_update);
        proto
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlaylistInfo {
    pub directories: Vec<String>,
    pub files: Vec<String>,
    pub total_files: u64,
    pub mode: ChangeMode,
    pub current_image: Option<String>,
    pub trigger_on_select: bool,
    pub use_last_on_select: bool,
    pub next_update: i64,
}

impl From<wcd::PlaylistInfo> for PlaylistInfo {
    fn from(proto: wcd::PlaylistInfo) -> PlaylistInfo {
        PlaylistInfo {
            directories: proto.directories.into_vec(),
            files: proto.files.into_vec(),
            total_files: proto.total_files,
            mode: proto.mode.into(),
            current_image: if proto.current_image.is_empty() { None } else { Some(proto.current_image) },
            trigger_on_select: proto.trigger_on_select,
            use_last_on_select: proto.use_last_on_select,
            next_update: proto.next_update,
        }
    }
}

impl Into<wcd::PlaylistInfo> for PlaylistInfo {
    fn into(self) -> wcd::PlaylistInfo {
        let mut proto = wcd::PlaylistInfo::new();
        proto.set_directories(::protobuf::RepeatedField::from_vec(self.directories));
        proto.set_files(::protobuf::RepeatedField::from_vec(self.files));
        proto.set_total_files(self.total_files);
        proto.set_mode(self.mode.into());
        proto.set_current_image(self.current_image.unwrap_or(String::new()));
        proto.set_trigger_on_select(self.trigger_on_select);
        proto.set_use_last_on_select(self.use_last_on_select);
        proto.set_next_update(self.next_update);
        proto
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ChangeMode {
    Sequential,
    Random,
}

impl From<wcd::ChangeMode> for ChangeMode {
    fn from(proto: wcd::ChangeMode) -> ChangeMode {
        match proto {
            wcd::ChangeMode::RANDOM => ChangeMode::Random,
            wcd::ChangeMode::SEQUENTIAL => ChangeMode::Sequential,
        }
    }
}

impl Into<wcd::ChangeMode> for ChangeMode {
    fn into(self) -> wcd::ChangeMode {
        match self {
            ChangeMode::Random => wcd::ChangeMode::RANDOM,
            ChangeMode::Sequential => wcd::ChangeMode::SEQUENTIAL,
        }
    }
}

impl From<config::ChangeMode> for ChangeMode {
    fn from(mode: config::ChangeMode) -> Self {
        match mode {
            config::ChangeMode::Sequential => ChangeMode::Sequential,
            config::ChangeMode::Random => ChangeMode::Random
        }
    }
}

pub trait GrpcResponseExt<T> {
    fn fold<F1, F2, U>(self, success: F1, failure: F2) -> ControlResponse
        where F1: FnOnce(U) -> ControlResponse,
              F2: FnOnce(String) -> ControlResponse,
              T: Into<U>;
}

impl<T> GrpcResponseExt<T> for (::grpc::Metadata, T, ::grpc::Metadata) {
    fn fold<F1, F2, U>(self, success: F1, failure: F2) -> ControlResponse
        where F1: FnOnce(U) -> ControlResponse,
              F2: FnOnce(String) -> ControlResponse,
              T: Into<U>
    {
        let (md, v, _) = self;
        match md.get("error") {
            Some(msg_bytes) => failure(String::from_utf8(msg_bytes.into()).unwrap()),
            None => success(v.into()),
        }
    }
}
