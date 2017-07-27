use std::thread::{self, JoinHandle};
use std::sync::{Arc, Barrier};
use std::iter::FromIterator;

use common::grpc::wcd;
use common::grpc::wcd_grpc::{WcdServer, Wcd};
use daemon::processor::Processor;
use daemon::stats::Stats;

pub struct Control {
    endpoint: String,
    daemon: super::Daemon,
}

impl Control {
    pub fn new(endpoint: String, daemon: super::Daemon) -> Control {
        Control {
            endpoint: endpoint,
            daemon: daemon,
        }
    }

    pub fn start(self) -> JoinHandle<()> {
        info!("Starting control thread");

        thread::spawn(move || {
            self.prepare_and_loop();
            self.daemon.scheduler().stop();
        })
    }

    fn prepare_and_loop(&self) {
        info!("Starting control server on {}", self.endpoint);

        let mut server = ::grpc::ServerBuilder::new_plain();
        if let Err(e) = server.http.set_addr(&self.endpoint) {
            error!("Setting listen address failed: {}", e);
            return;
        }
        server.http.set_cpu_pool_threads(1);

        let stop_barrier = Arc::new(Barrier::new(2));

        server.add_service(WcdServer::new_service_def(ControlServerImpl {
            processor: self.daemon.processor(),
            stats: self.daemon.stats(),
            stop_barrier: stop_barrier.clone(),
        }));

        let _server = server.build().expect("Creating the control server failed");

        info!("Control server started, waiting for requests");
        stop_barrier.wait();

        info!("Stopping control server");  // will stop upon drop

        struct ControlServerImpl {
            processor: Processor,
            stats: Option<Stats>,
            stop_barrier: Arc<Barrier>,
        }

        fn completed<T: Send + 'static>(t: T) -> ::grpc::SingleResponse<T> {
            ::grpc::SingleResponse::completed(t)
        }

        fn error<T: ::protobuf::MessageStatic>(msg: String) -> ::grpc::SingleResponse<T> {
            let mut md = ::grpc::Metadata::new();
            md.add(::grpc::MetadataKey::from("error"), msg.into_bytes().into());
            ::grpc::SingleResponse::completed_with_metadata(md, T::new())
        }

        impl Wcd for ControlServerImpl {
            fn trigger_change(&self, _: ::grpc::RequestOptions, _: wcd::Empty) -> ::grpc::SingleResponse<wcd::Empty> {
                match self.processor.trigger(true) {
                    Ok(_) => completed(wcd::Empty::new()),
                    Err(e) => error(e.to_string()),
                }
            }

            fn refresh_playlists(&self, _: ::grpc::RequestOptions, _: wcd::Empty) -> ::grpc::SingleResponse<wcd::Empty> {
                match self.processor.refresh_playlists(true) {
                    Ok(_) => completed(wcd::Empty::new()),
                    Err(e) => error(e.to_string()),
                }
            }

            fn terminate(&self, _: ::grpc::RequestOptions, _: wcd::Empty) -> ::grpc::SingleResponse<wcd::Empty> {
                self.stop_barrier.wait();
                completed(wcd::Empty::new())
            }

            fn get_status(&self, _: ::grpc::RequestOptions, _: wcd::Empty) -> ::grpc::SingleResponse<wcd::StatusInfo> {
                match self.processor.get_status() {
                    Ok(status_info) => completed(status_info.into()),
                    Err(e) => error(e.to_string()),
                }
            }

            fn change_playlist(&self, _: ::grpc::RequestOptions, p: wcd::PlaylistName) -> ::grpc::SingleResponse<wcd::Empty> {
                match self.processor.change_playlist(p.get_name()) {
                    Ok(_) => completed(wcd::Empty::new()),
                    Err(e) => error(e.to_string()),
                }
            }

            fn get_statistics(&self, _: ::grpc::RequestOptions, _: wcd::Empty) -> ::grpc::SingleResponse<wcd::StatsInfo> {
                let stats = match self.stats {
                    Some(ref stats) => stats,
                    None => return error("Statistics collection is disabled".into())
                };

                match stats.load() {
                    Ok(image_stats) => {
                        let mut proto = wcd::StatsInfo::new();
                        proto.set_image_stats(::protobuf::RepeatedField::from_iter(
                            image_stats.into_iter().map(Into::into)
                        ));
                        completed(proto)
                    }
                    Err(e) => error(e.to_string()),
                }
            }
        }
    }
}
