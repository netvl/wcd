extern crate docopt;
extern crate nanomsg;
extern crate bincode;
extern crate rustc_serialize;
extern crate wcd_common;

use std::io::Write;

use docopt::Docopt;
use nanomsg::{Socket, Protocol};
use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode_into};

use wcd_common::{util, config};
use wcd_common::proto::{self, ControlRequest, ControlResponse, ControlEnvelope};

const USAGE: &'static str = r"
Usage: wcdctl [options] ping
       wcdctl [options] trigger
       wcdctl [options] update
       wcdctl [options] terminate
       wcdctl [options] status
       wcdctl [options] set-playlist <name>
       wcdctl (--help | --version)

Options:
    -c FILE, --config FILE  path to configuration file [default: ~/.config/wcd/config.toml]
    -h, --help              show this message
    -v, --version           show version information
";

#[derive(RustcDecodable, Debug)]
struct Args {
    flag_help: bool,
    flag_version: bool,
    flag_config: String,

    arg_name: String,

    cmd_ping: bool,
    cmd_trigger: bool,
    cmd_update: bool,
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

    let req = if args.cmd_ping {
        ControlRequest::Ping       
    } else if args.cmd_trigger {
        ControlRequest::TriggerChange
    } else if args.cmd_update {
        ControlRequest::ForceUpdate
    } else if args.cmd_terminate {
        ControlRequest::Terminate
    } else if args.cmd_status {
        ControlRequest::GetStatus
    } else if args.cmd_set_playlist {
        ControlRequest::ChangePlaylist(args.arg_name)
    } else {
        abort!(1, "Unknown command");
    };

    let envelope = ControlEnvelope {
        version: proto::VERSION.into(),
        content: req
    };

    encode_into(&envelope, &mut socket, SizeLimit::Infinite)
        .unwrap_or_else(|e| abort!(1, "Error sending request: {}", e));

    let ControlEnvelope { version, content: resp } = decode_from(&mut socket, SizeLimit::Infinite)
        .unwrap_or_else(|e| abort!(1, "Error receiving response: {}", e));
    if version != proto::VERSION {
        abort!(1, "Received response with invalid version {}, expected {}", version, proto::VERSION);
    }
    let resp: ControlResponse = resp;

    println!("{:#?}", resp);

    ep.shutdown()
        .unwrap_or_else(|e| abort!(1, "Error closing socket endpoint: {}", e));
}