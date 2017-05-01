use std::io::Write;
use std::fmt;
use std::borrow::Cow;
use std::path::Path;

use clap::{App, AppSettings, SubCommand, Arg, ArgMatches};
use nanomsg::{Socket, Protocol};

use common::{util, config};
use common::proto::{self, ControlRequest, ControlResponse, ControlEnvelope, PlaylistInfo, ChangeMode};
use chrono::{Local, TimeZone};

macro_rules! abort {
    ($code:expr) => {::std::process::exit($code)};
    ($code:expr, $($args:tt)*) => {{
        let _ = writeln!(&mut ::std::io::stderr(), $($args)*);
        ::std::process::exit($code);
    }}
}

pub fn subcommands() -> Vec<App<'static, 'static>> {
    vec![
        SubCommand::with_name("trigger")
            .setting(AppSettings::ColoredHelp)
            .about("Triggers the wallpaper change in the current playlist"),
        SubCommand::with_name("refresh")
            .setting(AppSettings::ColoredHelp)
            .about("Makes wcd rescan all directories in all playlist, potentially loading new files"),
        SubCommand::with_name("terminate")
            .setting(AppSettings::ColoredHelp)
            .about("Shuts wcd down"),
        SubCommand::with_name("status")
            .setting(AppSettings::ColoredHelp)
            .about("Displays the current status information (available playlists, current items in them, timestamps, etc)"),
        SubCommand::with_name("set-playlist")
            .setting(AppSettings::ColoredHelp)
            .about("Sets the given playlist as the current one (may cause immediate wallpaper switch, depending on the selected playlist configuration)")
            .args_from_usage(
                "<NAME> 'Name of the playlist'
                 --or-trigger 'If the given playlist is already current, trigger the wallpaper change'"
            ),
    ]
}

pub fn main(config_path: Cow<Path>, subcommand: &str, matches: &ArgMatches) {
    let config = config::load(&config_path)
        .unwrap_or_else(|e| abort!(1, "Cannot load configuration file {}: {}", config_path.display(), e));

    let endpoint = config.common.endpoint;

    let mut socket = Socket::new(Protocol::Pair)
        .unwrap_or_else(|e| abort!(1, "Error creating nanomsg socket: {}", e));
    socket.set_send_timeout(1000)  // 1 second is more than enough
        .unwrap_or_else(|e| abort!(1, "Error setting socket timeout: {}", e));

    let mut ep = socket.connect(&endpoint)
        .unwrap_or_else(|e| abort!(1, "Error connecting socket to {}: {}", endpoint, e));

    let req = match subcommand {
        "trigger" => ControlRequest::TriggerChange,
        "refresh" => ControlRequest::RefreshPlaylists,
        "terminate" => ControlRequest::Terminate,
        "status" => ControlRequest::GetStatus,
        "set-playlist" => {
            let playlist_name = matches.value_of("NAME").unwrap();

            if matches.is_present("or-trigger") {
                match make_request(&mut socket, ControlRequest::GetStatus) {
                    ControlResponse::StatusInfo { current_playlist, .. } => {
                        if playlist_name == current_playlist {
                            ControlRequest::TriggerChange
                        } else {
                            ControlRequest::ChangePlaylist(playlist_name.into())
                        }
                    }
                    _ => abort!(1, "Unexpected server response when getting current playlist")
                }
            } else {
                ControlRequest::ChangePlaylist(playlist_name.into())
            }
        }
        _ => unreachable!()
    };

    display_response(make_request(&mut socket, req));

    ep.shutdown().unwrap_or_else(|e| abort!(1, "Error closing socket endpoint: {}", e));
}

fn make_request(socket: &mut Socket, req: ControlRequest) -> ControlResponse {
    let envelope = ControlEnvelope::wrap(req);

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
        ControlResponse::TriggerChangeOk | ControlResponse::RefreshPlaylistsOk |
        ControlResponse::TerminateOk | ControlResponse::ChangePlaylistOk => {}
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

struct TimestampDisplay(i64);

impl fmt::Display for TimestampDisplay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let dt = Local.timestamp(self.0, 0);
        dt.format("%F %T").fmt(f)
    }
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
