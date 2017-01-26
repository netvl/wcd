use std::collections::HashMap;
use std::io::{self, Read, Write};

use bincode::SizeLimit;
use bincode::serde::{serialize, deserialize, SerializeError, DeserializeError};
use serde::{Deserialize, Serialize};

use config;

pub const VERSION: u32 = 1;

quick_error! {
    #[derive(Debug)]
    pub enum ProtoError {
        Io(err: io::Error) {
            from()
            description("I/O error")
            display("I/O error: {}", err)
            cause(err)
        }
        Serialization(err: SerializeError) {
            from()
            description("serialization error")
            display("serialization error: {}", err)
            cause(err)
        }
        Deserialization(err: DeserializeError) {
            from()
            description("deserialization error")
            display("deserialization error: {}", err)
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

    StatusInfo {
        playlists: HashMap<String, PlaylistInfo>,
        current_playlist: String,
        last_update: i64
    },

    ChangePlaylistOk,
    ChangePlaylistFailed,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ControlEnvelope<T> {
    pub version: u32,
    pub content: T,
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

pub fn read_message<R: Read + ?Sized, T: Deserialize>(r: &mut R) -> Result<T, ProtoError> {
    // read the size and deserialize it into a big-endian u32
    let mut size_buf = [0u8; 4];
    if r.read(&mut size_buf)? < 4 {
        return Err(
            io::Error::new(io::ErrorKind::UnexpectedEof, "size message is too small").into()
        );
    }
    let size: u32 = ((size_buf[0] as u32) << 24) | 
                    ((size_buf[1] as u32) << 16) | 
                    ((size_buf[2] as u32) << 8) | 
                    size_buf[3] as u32;

    // read the data message
    let mut data_buf = vec![0u8; size as usize];
    let bytes_read = r.read(&mut data_buf)?;
    if bytes_read < size as usize {
        Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("data message is too small, read {} bytes, expected {} bytes", bytes_read, size)
        ).into())
    } else {
        // and decode it
        deserialize(&data_buf).map_err(From::from)
    }
}

pub fn write_message<W: Write + ?Sized, T: Serialize>(w: &mut W, value: &T) -> Result<(), ProtoError> {
    // first encode the data message
    let data = serialize(value, SizeLimit::Infinite)?;

    // then compute a big-endian size representation and write it
    let size = data.len() as u32;
    let size_buf = [
        ((size >> 24) & 0xFF) as u8,
        ((size >> 16) & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        (size & 0xFF) as u8
    ];
    w.write_all(&size_buf).map_err(ProtoError::from)?;

    // write the data message
    w.write_all(&data).map_err(From::from)
}
