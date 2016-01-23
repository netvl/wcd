use std::thread::{self, JoinHandle};

use chan::{self, Receiver, Sender};
use nanomsg::{Socket, Protocol};
use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, encode_into};

use wcd_common::proto::{ControlRequest, ControlResponse};

pub fn start(endpoint: String) -> (Receiver<ControlRequest>, Sender<ControlResponse>, JoinHandle<()>) {
    let (control_req_send, control_req_recv) = chan::sync(0);
    let (control_resp_send, control_resp_recv) = chan::sync(0);

    let jh = thread::spawn(move || {
        let mut socket = match Socket::new(Protocol::Pair) {
            Ok(socket) => socket,
            Err(e) => {
                warn!("Cannot create control socket: {}", e);
                return;
            }
        };

        let _ep = match socket.bind(&endpoint) {
            Ok(ep) => ep,
            Err(e) => {
                warn!("Error binding control socket to {}: {}", endpoint, e);
                return;
            }
        };

        info!("Control socket created on {}, waiting for requests", endpoint);

        loop {
            match decode_from(&mut socket, SizeLimit::Infinite) {
                Ok(req) => {
                    debug!("Received request from a client: {:?}", req);
                    control_req_send.send(req);

                    match control_resp_recv.recv() {
                        Some(resp) => {
                            match encode_into(&resp, &mut socket, SizeLimit::Infinite) {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Error writing response: {}", e);
                                }
                            }
                        }
                        None => {
                            warn!("Server channel closed unexpectedly, stopping the control socket");
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading request: {}", e);
                }
            }
        }
    });

    (control_req_recv, control_resp_send, jh)
}
