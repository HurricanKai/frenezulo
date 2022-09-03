use lunatic::{abstract_process, process::ProcessRef, Tag, Process, Mailbox};
use submillisecond::{Application, RequestContext, http::{Response, Version}, params::Param};

use super::router;

pub struct Listener(Process<()>);

#[abstract_process]
impl Listener {
    fn handler(context : RequestContext) -> Response<Vec<u8>> {
        let request = context.request;

        let path = request.uri().path();
        let prefix = match path.split_once('/') {
            Some(("", rest)) => match rest.split_once('/') {
                Some((prefix, _rest)) => Some(prefix),
                _ => None
            },
            Some((prefix, _rest)) => Some(prefix),
            _ => None
        };
        
        todo!("spawn router process & wait for it");

        Response::builder()
            .version(request.version())
            .status(404)
            .body(format!("Unknown Service {}", path).as_bytes().to_owned())
            .expect("builder has to succeed")
    }

    #[init]
    fn init(_: ProcessRef<Self>, _: ()) -> Self {
        let process = Process::spawn_link((), |(), _:Mailbox<()>| {
            Application
                ::new(Self::handler as fn(_) -> _)
                .serve("0.0.0.0:3000")
                .expect("failed to start server");
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