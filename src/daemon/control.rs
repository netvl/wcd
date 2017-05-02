use std::thread::{self, JoinHandle};
use std::time::Duration as StdDuration;
use std::io;

use nanomsg::{Socket, Protocol};

use common::proto::{self, ControlRequest, ControlResponse, ControlEnvelope, ProtoError};
use daemon::processor::Processor;
use daemon::scheduler::Scheduler;

pub struct Control {
    endpoint: String,
    processor: Processor,
    scheduler: Scheduler,
}

impl Control {
    pub fn new(endpoint: String, processor: Processor, scheduler: Scheduler) -> Control {
        Control {
            endpoint: endpoint,
            processor: processor,
            scheduler: scheduler,
        }
    }

    pub fn start(mut self) -> JoinHandle<()> {
        info!("Starting the control thread");

        thread::spawn(move || {
            self.prepare_and_loop();
            self.scheduler.stop();
        })
    }

    fn prepare_and_loop(&mut self) {
        let mut socket = match Socket::new(Protocol::Pair) {
            Ok(socket) => socket,
            Err(e) => {
                warn!("Cannot create a control socket: {}", e);
                return;
            }
        };

        match socket.set_receive_timeout(1000) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to set the control socket receive timeout: {}", e);
                return;
            }
        }

        match socket.set_send_timeout(3000) {
            Ok(_) => {},
            Err(e) => {
                error!("Failed to set the control socket send timeout: {}", e);
                return;
            }
        }

        let mut ep = match socket.bind(&self.endpoint) {
            Ok(ep) => ep,
            Err(e) => {
                error!("Failed to bind the control socket to {}: {}", self.endpoint, e);
                return;
            }
        };

        info!("Control socket created on {}, waiting for requests", self.endpoint);

        loop {
            match proto::read_message(&mut socket) {
                Ok(ControlEnvelope { version, content: req }) => {
                    if version != proto::VERSION {
                        warn!("Received a control request with invalid version {}, expected {}", version, proto::VERSION);
                        continue;
                    }

                    debug!("Received a request from a client: {:?}", req);
                    let mut should_break = false;
                    let resp = match req {
                        ControlRequest::TriggerChange => {
                            let _ = self.processor.trigger(true);
                            ControlResponse::TriggerChangeOk
                        }
                        ControlRequest::RefreshPlaylists => {
                            let _ = self.processor.refresh_playlists(true);
                            ControlResponse::RefreshPlaylistsOk
                        }
                        ControlRequest::Terminate => {
                            should_break = true;
                            ControlResponse::TerminateOk
                        }
                        ControlRequest::GetStatus => {
                            match self.processor.get_status() {
                                Ok(status_info) => ControlResponse::StatusInfoOk(status_info),
                                Err(_) => ControlResponse::StatusInfoFailed,
                            }
                        }
                        ControlRequest::ChangePlaylist(name) => {
                            match self.processor.change_playlist(&name) {
                                Ok(_) => ControlResponse::ChangePlaylistOk,
                                Err(_) => ControlResponse::ChangePlaylistFailed,
                            }
                        }
                    };

                    let envelope = ControlEnvelope::wrap(resp);
                    match proto::write_message(&mut socket, &envelope) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("Error writing response: {}", e);
                        }
                    }

                    if should_break {
                        break;
                    }
                }
                Err(ProtoError::Io(ref e)) if e.kind() == io::ErrorKind::TimedOut => {
                    // this is fine, continue
                }
                Err(e) => {
                    error!("Error reading request: {}", e);
                    thread::sleep(StdDuration::from_secs(1));  // just in case
                }
            }
        }

        info!("Stopping the control thread");

        if let Err(e) = ep.shutdown() {
            warn!("Failed to shut down socket endpoint: {}", e);
        }
    }
}
