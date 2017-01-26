use std::thread::{self, JoinHandle};
use std::time::Duration as StdDuration;
use std::io;

use chan::{self, Receiver, Sender};
use nanomsg::{Socket, Protocol};

use wcd_common::proto::{self, ControlRequest, ControlResponse, ControlEnvelope, ProtoError};

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

        match socket.set_receive_timeout(1000) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to set control socket receive timeout: {}", e);
                return;
            }
        }

        match socket.set_send_timeout(3000) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to set control socket send timeout: {}", e);
                return;
            }
        }

        let mut ep = match socket.bind(&endpoint) {
            Ok(ep) => ep,
            Err(e) => {
                error!("Failed to bind control socket to {}: {}", endpoint, e);
                return;
            }
        };

        info!("Control socket created on {}, waiting for requests", endpoint);

        loop {
            match proto::read_message(&mut socket) {
                Ok(ControlEnvelope { version, content: req }) => {
                    if version == proto::VERSION {
                        debug!("Received request from a client: {:?}", req);
                        control_req_send.send(req);

                        match control_resp_recv.recv() {
                            Some(resp) => {
                                let envelope = ControlEnvelope {
                                    version: proto::VERSION.into(),
                                    content: resp
                                };
                                match proto::write_message(&mut socket, &envelope) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        error!("Error writing response: {}", e);
                                    }
                                }
                            }
                            None => {
                                error!("Control channel was broken unexpectedly, exiting");
                                break;
                            }
                        }
                    } else {
                        warn!("Received control request with invalid version {}, expected {}", version, proto::VERSION);
                    }
                }
                Err(ProtoError::Io(ref e)) if e.kind() == io::ErrorKind::TimedOut => {
                    chan_select! {
                        default => {},  // okay, do nothing
                        control_resp_recv.recv() => {  
                            // we can only get here if the remote end of the channel was dropped
                            info!("Control channel was broken, exiting");
                            break;
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading request: {}", e);
                    thread::sleep(StdDuration::from_secs(1));  // just in case
                }
            }
        }

        if let Err(e) = ep.shutdown() {
            warn!("Failed to shut down socket endpoint: {}", e);
        }
    });

    (control_req_recv, control_resp_send, jh)
}
