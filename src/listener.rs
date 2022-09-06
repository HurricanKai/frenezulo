use std::time::Duration;

use lunatic::{abstract_process, process::ProcessRef, Tag, Process, Mailbox, spawn_link};
use serde::{Serialize, Deserialize};
use submillisecond::{Application, RequestContext, http::Response, Handler, headers::HeaderMapExt};

use crate::{service_registry, router};

pub struct Listener(Process<()>);

#[derive(Serialize, Deserialize, Clone, Copy)]
struct AppHandler {

}

impl Handler for AppHandler {
    fn handle(&self, context: RequestContext) -> submillisecond::response::Response {
        let request = context.request;
        // let mailbox : Mailbox<crate::http::Response> = context.mailbox;
        let mailbox : Mailbox<crate::http::Response> = unsafe { Mailbox::new() };

        let path = request.uri().path();
        let prefix = match path.split_once('/') {
            Some(("", rest)) => match rest.split_once('/') {
                Some((prefix, _rest)) => Some(prefix),
                _ => None
            },
            Some((prefix, _rest)) => Some(prefix),
            _ => None
        };
        
        let mut response = match prefix {
            Some(prefix) => match router::create_request(prefix.to_owned()) {
                Some((service_id, request_id)) =>{
                    let (m, b) = request.into_parts();
                    let req = frenezulo::http::Request
                    {
                        metadata: m.into(),
                        body: serde_bytes::ByteBuf::from(b.as_slice().to_vec())
                    };
                    
                    service_registry::start_request(request_id, service_id, req, Process::this());
                    match mailbox.receive_timeout(Duration::from_secs(30)) {
                        lunatic::MailboxResult::Message(response) => response.into(),
                        lunatic::MailboxResult::DeserializationFailed(_) => todo!(),
                        lunatic::MailboxResult::TimedOut =>
                            Response::builder()
                                .status(408)
                                .body(b"Outer 30s timeout has been hit. This should never happen.".to_vec())
                                .expect("Timeout builder has to succeed"),
                        lunatic::MailboxResult::LinkDied(_) => todo!(),
                    }
                },
                None => Response::builder()
                        .version(request.version())
                        .status(404)
                        .body(b"Unknown Service".to_vec()).expect("404 builder has to succeed")
            },
            None => Response::builder()
                    .version(request.version())
                    .status(404)
                    .body(b"Path did not include service prefix".to_vec()).expect("404 builder has to succeed")
        };

        // for testing: restart requests after each HTTP request
        response.headers_mut().typed_insert(submillisecond::headers::Connection::close());

        response
    }
}

#[abstract_process]
impl Listener {
    #[init]
    fn init(_: ProcessRef<Self>, _: ()) -> Self {
        let process = spawn_link!(|| {
            Application::new_custom(AppHandler { }).serve("0.0.0.0:3000").expect("Server has to start");
        });
        Self(process)
    }

    #[terminate]
    fn terminate(self) {
        self.0.kill()
    }

    #[handle_link_trapped]
    fn handle_link_trapped(&self, _tag: Tag) {
        println!("Link trapped");
    }
}