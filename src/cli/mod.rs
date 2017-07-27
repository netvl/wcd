use std::io::Write;
use std::fmt;
use std::borrow::Cow;
use std::path::Path;

use clap::{App, ArgMatches};
use chrono::{Local, TimeZone};

use common::config;
use common::proto::{ControlRequest, ControlResponse, StatusInfo, PlaylistInfo, ChangeMode};
use self::client::Client;

mod client;

macro_rules! abort {
    ($code:expr) => {::std::process::exit($code)};
    ($code:expr, $($args:tt)*) => {{
        let _ = writeln!(&mut ::std::io::stderr(), $($args)*);
        ::std::process::exit($code);
    }}
}

pub fn subcommands() -> Vec<App<'static, 'static>> {
    vec![
        App::new("trigger")
            .about("Triggers the wallpaper change in the current playlist"),
        App::new("refresh")
            .about("Makes wcd rescan all directories in all playlist, potentially loading new files"),
        App::new("terminate")
            .about("Shuts wcd down"),
        App::new("status")
            .about("Displays the current status information (available playlists, current items in them, timestamps, etc)"),
        App::new("set-playlist")
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

    let client = client::Client::new(&endpoint)
        .unwrap_or_else(|e| abort!(1, "Error creating a gRPC client: {}", e));

    let req = match subcommand {
        "trigger" => ControlRequest::TriggerChange,
        "refresh" => ControlRequest::RefreshPlaylists,
        "terminate" => ControlRequest::Terminate,
        "status" => ControlRequest::GetStatus,
        "set-playlist" => {
            let playlist_name = matches.value_of("NAME").unwrap();

            if matches.is_present("or-trigger") {
                match make_request(&client, ControlRequest::GetStatus) {
                    ControlResponse::StatusInfoOk(StatusInfo { current_playlist, .. }) => {
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

    display_response(make_request(&client, req));
}

fn make_request(client: &Client, req: ControlRequest) -> ControlResponse {
    client.send(req)
        .unwrap_or_else(|e| abort!(1, "Error sending request: {}", e))
}

fn display_response(resp: ControlResponse) {
    match resp {
        ControlResponse::TriggerChangeOk | ControlResponse::RefreshPlaylistsOk |
        ControlResponse::TerminateOk | ControlResponse::ChangePlaylistOk => {}
        ControlResponse::ChangePlaylistFailed(msg) => abort!(1, "Failed to change playlist: {}", msg),
        ControlResponse::StatusInfoOk(StatusInfo { playlists, current_playlist, last_update }) => {
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
        ControlResponse::StatusInfoFailed(msg) => abort!(1, "Failed to retrieve status info: {}", msg)
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
