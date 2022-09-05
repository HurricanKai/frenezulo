use lunatic::{Mailbox, process::StartProcess};
use crate::application::Application;
mod http;
mod module_supervisor;
mod service_registry;
mod listener;
mod router;
mod application;

/*
fn index() -> &'static str {
    "Hello :)"
}

fn headers(headers : HeaderMap) -> String {
    headers
        .iter()
        .map(|(name, val)| -> String {
            let header_string_value = val.to_str();
            let header_value = match header_string_value {
                Ok(string) => string.to_owned(),
                Err(_) => format!("<binary> l={}", val.len()),
            };
            format!("{}: {}\n", name.as_str(), header_value)
        })
        .fold(String::new(), |mut a : String, b| {
            a.reserve(b.len() + 1);
            a.push_str(&b);
            a.push('\n');
            a
        })
}*/

/*
struct Service404;
impl Service for Service404 {
    fn request(&self, request : Request<Body>) -> Response<Vec<u8>> {
        Response::builder()
                .version(request.version())
                .status(404)
                .body("404!".as_bytes())
                .expect("builder has to succeed")
    }
}*/

/*
fn handler(req: RequestContext) -> Response<Vec<u8>> {
    let request = req.request;
    Response::builder()
        .version(request.version())
        .status(404)
        .body(format!("Unknown Service {}", request.uri().path()).into_bytes())
        .expect("builder has to succeed")
}*/

fn start_app() {
    Application::start_link((), None);
    router::add_service("test1".to_owned(), std::fs::read("./test.wasm").expect("File has to exist"));
    router::add_service("test2".to_owned(), std::fs::read("./test.wasm").expect("File has to exist"));
    router::add_service("test3".to_owned(), std::fs::read("./test.wasm").expect("File has to exist"));
    router::add_service("test4".to_owned(), std::fs::read("./test.wasm").expect("File has to exist"));
    router::add_service("test5".to_owned(), std::fs::read("./test.wasm").expect("File has to exist"));
}

#[lunatic::main]
fn main(mailbox: Mailbox<()>) {
    start_app();

    let failure_mailbox = mailbox.catch_link_failure();
    loop {
        let message = failure_mailbox.receive();
        assert!(message.is_link_died());
        start_app();
    }
}