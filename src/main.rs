use lunatic::{Mailbox, process::StartProcess};

use crate::application::Application;
mod encoding;
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
    Application::start((), None).link()
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