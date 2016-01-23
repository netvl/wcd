
#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq)]
pub enum ControlRequest {
    Ping,
    TriggerChange,
    Terminate,
}

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq)]
pub enum ControlResponse {
    PingOk,
    TriggerChangeOk,
    TerminateOk,
}
