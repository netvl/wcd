pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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

#[derive(RustcDecodable, RustcEncodable, Debug, Clone, PartialEq, Eq)]
pub struct ControlEnvelope<T> {
    pub version: String,
    pub content: T
}
