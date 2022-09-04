use lunatic::{Tag, Process};
use serde::{Serialize, Deserialize};

pub mod http;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ServiceId {
    pub tag: Tag
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct RequestId {
    pub tag: Tag
}

pub mod module_supervisor {
    use serde::{Serialize, Deserialize};

    #[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ModuleSupervisorMessage {
        CompleteRequest(crate::RequestId, crate::http::Response)
    }

    pub type WorkerSerializer = lunatic::serializer::Json;
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerMessage {
    Request(RequestId, crate::http::Request, Process<module_supervisor::ModuleSupervisorMessage, crate::module_supervisor::WorkerSerializer>),
}
