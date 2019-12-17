use std::error::Error;
use std::result;

use grpc::ClientStubExt;

use crate::common::proto::GrpcResponseExt;
use crate::common::grpc::wcd;
use crate::common::grpc::wcd_grpc::{WcdClient, Wcd};
use crate::common::proto::{ControlRequest, ControlResponse};

type Result<T> = result::Result<T, Box<dyn Error>>;

pub struct Client {
    grpc: WcdClient,
}

impl Client {
    pub fn new(endpoint: &str) -> Result<Client> {
        let mut parts = endpoint.split(":");
        let host = parts.next();
        let port = parts.next();

        let (host, port) = match (host, port) {
            (Some(host), Some(port)) => (host, port),
            _ => unimplemented!()
        };

        let grpc = WcdClient::new_plain(host, port.parse().expect("Endpoint port is invalid"), Default::default())?;

        Ok(Client { grpc, })
    }

    pub fn send(&self, msg: ControlRequest) -> Result<ControlResponse> {
        match msg {
            ControlRequest::TriggerChange =>
                Ok(self.grpc.trigger_change(Default::default(), wcd::Empty::new()).wait_drop_metadata()
                    .map(|_| ControlResponse::TriggerChangeOk)?),
            ControlRequest::TriggerUpdate =>
                Ok(self.grpc.trigger_update(Default::default(), wcd::Empty::new()).wait_drop_metadata()
                    .map(|_| ControlResponse::TriggerUpdateOk)?),
            ControlRequest::RefreshPlaylists =>
                Ok(self.grpc.refresh_playlists(Default::default(), wcd::Empty::new()).wait_drop_metadata()
                    .map(|_| ControlResponse::RefreshPlaylistsOk)?),
            ControlRequest::Terminate =>
                Ok(self.grpc.terminate(Default::default(), wcd::Empty::new()).wait_drop_metadata()
                    .map(|_| ControlResponse::TerminateOk)?),
            ControlRequest::GetStatus => {
                Ok(self.grpc.get_status(Default::default(), wcd::Empty::new()).wait()?
                    .fold(ControlResponse::StatusInfoOk, ControlResponse::StatusInfoFailed))
            }
            ControlRequest::ChangePlaylist(playlist) => {
                let mut playlist_name = wcd::PlaylistName::new();
                playlist_name.set_name(playlist);
                Ok(self.grpc.change_playlist(Default::default(), playlist_name).wait()?
                    .fold(|_: wcd::Empty| ControlResponse::ChangePlaylistOk, ControlResponse::ChangePlaylistFailed))
            }
        }
    }
}
