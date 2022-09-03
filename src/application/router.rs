use lunatic::Tag;
use serde::{Serialize, Deserialize};

use crate::http::Response;

mod supervisor;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct ServiceId {
    pub tag: Tag
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct RequestId {
    pub tag: Tag
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
enum RouterMessage {
    Response(Response)
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct RouterInfo {

}

/*

        Response::builder()
            .version(request.version())
            .status(404)
            .body(format!("Unknown Service {}", path).as_bytes().to_owned())
            .expect("builder has to succeed")
*/