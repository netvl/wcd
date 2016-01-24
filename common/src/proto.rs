use std::collections::HashMap;
use std::io::{self, Read, Write};

use bincode::SizeLimit;
use bincode::rustc_serialize::{DecodingResult, DecodingError, decode, EncodingResult, EncodingError, encode};
use rustc_serialize::{Decodable, Encodable};

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
    pub directories: Vec<String>,
    pub files: Vec<String>,
    pub total_files: u64,
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

pub fn read_message<R: Read + ?Sized, T: Decodable>(r: &mut R) -> DecodingResult<T> {
    // read the size and deserialize it into a big-endian u32
    let mut size_buf = [0u8; 4];
    if try!(r.read(&mut size_buf)) < 4 {
        return Err(DecodingError::IoError(
            io::Error::new(io::ErrorKind::UnexpectedEof, "size message is too small")
        ));
    }
    let size: u32 = ((size_buf[0] as u32) << 24) | 
                    ((size_buf[1] as u32) << 16) | 
                    ((size_buf[2] as u32) << 8) | 
                    size_buf[3] as u32;

    // read the data message
    let mut data_buf = vec![0u8; size as usize];
    let bytes_read = try!(r.read(&mut data_buf));
    if bytes_read < size as usize {
        Err(DecodingError::IoError(
            io::Error::new(
                io::ErrorKind::UnexpectedEof, 
                format!("data message is too small: {}, expected {}", bytes_read, size)
            )
        ))
    } else {
        // and decode it
        decode(&data_buf)
    }
}

pub fn write_message<W: Write + ?Sized, T: Encodable>(w: &mut W, value: &T) -> EncodingResult<()> {
    // first encode the data message
    let data = try!(encode(value, SizeLimit::Infinite));

    // compute a big-endian size representation and write it
    let size = data.len() as u32;
    let size_buf = [
        ((size >> 24) & 0xFF) as u8,
        ((size >> 16) & 0xFF) as u8,
        ((size >> 8) & 0xFF) as u8,
        (size & 0xFF) as u8
    ];
    try!(w.write_all(&size_buf).map_err(EncodingError::IoError));

    // write the data message
    w.write_all(&data).map_err(EncodingError::IoError)
}
