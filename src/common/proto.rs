use std::collections::HashMap;
use std::io::{self, Read, Write};

use bincode::{self, serialize, deserialize, Infinite};
use serde::{Deserialize, Serialize};

use common::config;

pub const VERSION: u32 = 2;

quick_error! {
    #[derive(Debug)]
    pub enum ProtoError {
        Io(err: io::Error) {
            from()
            description("I/O error")
            display("I/O error: {}", err)
            cause(err)
        }
        Bincode(err: bincode::Error) {
            from()
            description("bincode (de)serialization error")
            display("bincode (de)serialization error: {}", err)
            cause(err)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ControlRequest {
    TriggerChange,
    RefreshPlaylists,
    Terminate,

    GetStatus,

    ChangePlaylist(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ControlResponse {
    TriggerChangeOk,
    RefreshPlaylistsOk,
    TerminateOk,

    StatusInfoOk(StatusInfo),
    StatusInfoFailed,

    ChangePlaylistOk,
    ChangePlaylistFailed,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ControlEnvelope<T> {
    pub version: u32,
    pub content: T,
}

impl<T> ControlEnvelope<T> {
    pub fn wrap(value: T) -> ControlEnvelope<T> {
        ControlEnvelope {
            version: VERSION,
            content: value,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StatusInfo {
    pub playlists: HashMap<String, PlaylistInfo>,
    pub current_playlist: String,
    pub last_update: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ChangeMode {
    Sequential,
    Random,
}

impl From<config::ChangeMode> for ChangeMode {
    fn from(mode: config::ChangeMode) -> Self {
        match mode {
            config::ChangeMode::Sequential => ChangeMode::Sequential,
            config::ChangeMode::Random => ChangeMode::Random
        }
    }
}

pub fn read_message<R: Read + ?Sized, T: for<'a> Deserialize<'a>>(r: &mut R) -> Result<T, ProtoError> {
    // read the size and deserialize it into a big-endian u32
    let mut size_buf = [0u8; 4];
    r.read_exact(&mut size_buf)?;
    let size: u32 = ((size_buf[0] as u32) << 24) |
                    ((size_buf[1] as u32) << 16) | 
                    ((size_buf[2] as u32) << 8) | 
                    size_buf[3] as u32;

    // read the data message
    let mut data_buf = vec![0u8; size as usize];
    r.read_exact(&mut data_buf)?;

    // and decode it
    Ok(deserialize(&data_buf)?)
}

pub fn write_message<W: Write + ?Sized, T: Serialize>(w: &mut W, value: &T) -> Result<(), ProtoError> {
    // first encode the data message
    let data = serialize(value, Infinite)?;

    // then compute a big-endian size representation and write it
    let size = data.len() as u32;
    let size_buf = [
        ((size >> 24) & 0xFF) as u8,
        ((size >> 16) & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        (size & 0xFF) as u8
    ];
    w.write_all(&size_buf)?;

    // write the data message
    Ok(w.write_all(&data)?)
}
