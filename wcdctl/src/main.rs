extern crate docopt;
extern crate nanomsg;
extern crate bincode;
extern crate rustc_serialize;
extern crate wcd_common;
extern crate chrono;

use std::io::Write;
use std::fmt;

use docopt::Docopt;
use nanomsg::{Socket, Protocol};

use wcd_common::{util, config};
use wcd_common::proto::{self, ControlRequest, ControlResponse, ControlEnvelope, PlaylistInfo, ChangeMode};
use chrono::{Local, TimeZone};

const USAGE: &'static str = r"
Usage: wcdctl [options] trigger
       wcdctl [options] refresh
       wcdctl [options] terminate
       wcdctl [options] status
       wcdctl [options] set-playlist [--or-trigger] <name>
       wcdctl (--help | --version)

Options:
    -c FILE, --config FILE  path to configuration file [default: ~/.config/wcd/config.toml]
    --or-trigger            if the given playlist is already current, trigger the wallpaper
                            change
    -h, --help              show this message
    -v, --version           show version information

Commands:
    trigger               triggers the wallpaper change in the current playlist
    refresh               makes wcd rescan all directories in all playlist, potentially 
                          loading new files
    terminate             shuts wcd down
    status                displays current status information (available playlists, 
                          current items in them, timestamps, etc.)
    set-playlist <name>   sets the provided playlist as the current one (may cause immediate
                          wallpaper switch, depending on the selected playlist configuration)
";

#[derive(RustcDecodable, Debug)]
struct Args {
    flag_help: bool,
    flag_version: bool,
    flag_config: String,
    flag_or_trigger: bool,

    arg_name: String,

    cmd_trigger: bool,
    cmd_refresh: bool,
    cmd_terminate: bool,
    cmd_status: bool,
    cmd_set_playlist: bool,
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

macro_rules! abort {
    ($code:expr) => {::std::process::exit($code)};
    ($code:expr, $($args:tt)*) => {{
        let _ = writeln!(&mut ::std::io::stderr(), $($args)*);
        ::std::process::exit($code);
    }}
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .unwrap_or_else(|e| e.exit())
        .help(true)
        .version(Some(VERSION.into()))
        .decode()
        .unwrap_or_else(|e| e.exit());

    let config_path = util::str_to_path(&args.flag_config);
    
    let config = config::load(&config_path)
        .unwrap_or_else(|e| abort!(1, "Cannot load configuration file {}: {}", config_path.display(), e));

    let endpoint = config.common.endpoint;

    let mut socket = Socket::new(Protocol::Pair)
        .unwrap_or_else(|e| abort!(1, "Error creating nanomsg socket: {}", e));
    socket.set_send_timeout(1000)  // 1 second is more than enough
        .unwrap_or_else(|e| abort!(1, "Error setting socket timeout: {}", e));

    let mut ep = socket.connect(&endpoint)
        .unwrap_or_else(|e| abort!(1, "Error connecting socket to {}: {}", endpoint, e));

    if !args.flag_or_trigger {
        let req = if args.cmd_trigger {
            ControlRequest::TriggerChange
        } else if args.cmd_refresh {
            ControlRequest::RefreshPlaylists
        } else if args.cmd_terminate {
            ControlRequest::Terminate
        } else if args.cmd_status {
            ControlRequest::GetStatus
        } else if args.cmd_set_playlist {
            ControlRequest::ChangePlaylist(args.arg_name)
        } else {
            abort!(1, "Unknown command");
        };

        display_response(make_request(&mut socket, req));

    } else {  // change playlist or trigger change, if needed
        match make_request(&mut socket, ControlRequest::GetStatus) {
            ControlResponse::StatusInfo { current_playlist, .. } => {
                let req = if args.arg_name == current_playlist {
                    ControlRequest::TriggerChange
                } else {
                    ControlRequest::ChangePlaylist(args.arg_name)
                };

                display_response(make_request(&mut socket, req))
            }
            _ => abort!(1, "Unexpected server response")
        }
    }

    ep.shutdown()
        .unwrap_or_else(|e| abort!(1, "Error closing socket endpoint: {}", e));
}

fn make_request(socket: &mut Socket, req: ControlRequest) -> ControlResponse {
    let envelope = ControlEnvelope {
        version: proto::VERSION.into(),
        content: req
    };

    proto::write_message(socket, &envelope)
        .unwrap_or_else(|e| abort!(1, "Error sending request: {}", e));

    let ControlEnvelope { version, content: resp } = proto::read_message(socket)
        .unwrap_or_else(|e| abort!(1, "Error receiving response: {}", e));
    if version != proto::VERSION {
        abort!(1, "Received response with invalid version {}, expected {}", version, proto::VERSION);
    }

    resp
}

fn display_response(resp: ControlResponse) {
    match resp {
        ControlResponse::TriggerChangeOk => {}
        ControlResponse::RefreshPlaylistsOk => {}
        ControlResponse::TerminateOk => {}
        ControlResponse::ChangePlaylistOk => {}
        ControlResponse::ChangePlaylistFailed => abort!(1, "Failed to change playlist"),
        ControlResponse::StatusInfo { playlists, current_playlist, last_update } => {
            println!("Last change time: {}", TimestampDisplay(last_update));
            println!("Current playlist: {}", current_playlist);

            if !playlists.is_empty() {
                println!("Playlists:");
                for (name, playlist) in playlists {
                    print_playlist(name, playlist);
                }
            } else {
                println!("No playlists available");
            }
        }
    }
}

struct TimestampDisplay(i64);

impl fmt::Display for TimestampDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dt = Local.timestamp(self.0, 0);
        dt.format("%F %T").fmt(f)
    }
}

fn print_playlist(name: String, playlist: PlaylistInfo) {
    println!("  {}:", name);
    if !playlist.files.is_empty() {
        println!("    Configured files:");
        for file in playlist.files {
            println!("      {}", file);
        }
    }
    if !playlist.directories.is_empty() {
        println!("    Configured directories:");
        for dir in playlist.directories {
            println!("      {}", dir);
        }
    }
    println!("    Total files number: {}", playlist.total_files);
    println!("    Change mode: {}", match playlist.mode {
        ChangeMode::Sequential => "sequential",
        ChangeMode::Random => "random",
    });
    println!("    Current image: {}", OptionDisplay(playlist.current_image));
    println!("    Trigger wallpaper change on select: {}", BoolDisplay(playlist.trigger_on_select));
    if playlist.trigger_on_select {
        println!("    Use last used wallpaper on select: {}", BoolDisplay(playlist.use_last_on_select));
    }
    println!("    Next change time: {}", TimestampDisplay(playlist.next_update));
}

struct OptionDisplay<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for OptionDisplay<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Some(ref v) => v.fmt(f),
            None => f.write_str("none"),
        }
    }
}

struct BoolDisplay(bool);

impl fmt::Display for BoolDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(if self.0 { "enabled" } else { "disabled" })
    }
}
