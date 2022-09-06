use frenezulo::{module_supervisor::ModuleSupervisorMessage, http::{Response, ResponseMetadata}};
use lunatic::{Mailbox};


#[export_name = "random_bullshit_go"]
extern "C" fn random_bullshit_go() {
    run(unsafe { Mailbox::<frenezulo::WorkerMessage, frenezulo::module_supervisor::WorkerSerializer>::new() })
}

#[lunatic::main]
fn main(mailbox: Mailbox::<frenezulo::WorkerMessage, frenezulo::module_supervisor::WorkerSerializer>) {
    run(mailbox)
}

fn run(mailbox: Mailbox<frenezulo::WorkerMessage, frenezulo::module_supervisor::WorkerSerializer>) {
    match mailbox.receive() {
        //MailboxResult::Message(msg) => match msg {
            frenezulo::WorkerMessage::Request(request_id, request, respond_to) => {
                let response = Response { metadata: ResponseMetadata { status: 200, version: request.metadata.version, headers: Default::default() },
                    body: b"Hello World!".to_vec()
                };
                respond_to.send(ModuleSupervisorMessage::CompleteRequest(
                    request_id, response,
                ));
            }
        //},
    }
}