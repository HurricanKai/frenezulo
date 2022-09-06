use lunatic::{Tag, Process};
use serde::{Serialize, Deserialize};

mod http;
pub use http::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ServiceId {
    pub tag: Tag
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct RequestId {
    pub tag: Tag
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleSupervisorMessage {
    CompleteRequest(crate::RequestId, crate::http::Response)
}

pub type WorkerSerializer = lunatic::serializer::MessagePack;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerMessage {
    Request(RequestId, crate::http::Request, Process<ModuleSupervisorMessage, WorkerSerializer>),
}
