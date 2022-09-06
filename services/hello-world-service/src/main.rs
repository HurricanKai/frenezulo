use frenezulo::{Response, ResponseMetadata, Request};

#[frenezulo::handler]
fn handle(request: Request) -> Response {
    Response { metadata: ResponseMetadata { status: 200, version: request.metadata.version, headers: Default::default() },
        body: b"Hello World!".to_vec()
    }
}