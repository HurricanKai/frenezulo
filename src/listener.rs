use std::{time::Duration, io::{Write, Read, BufReader, BufRead}};

use lunatic::{abstract_process, process::ProcessRef, Tag, Process, Mailbox, spawn_link, net::TcpStream};
use serde::{Serialize, Deserialize};
use submillisecond::{Application, RequestContext, http::{Response, Uri}, Handler, Json, extract::FromRequest};
use anyhow::anyhow;

use crate::{service_registry::{self}, router};

pub struct Listener(Process<()>);

#[derive(Serialize, Deserialize, Clone, Copy)]
struct AppHandler {

}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ServiceAdd {
    prefix: String,
    source: String,
}

fn service_add(request: &mut RequestContext) -> anyhow::Result<Response<Vec<u8>>> {
    println!("service_add");
    let data = Json::<ServiceAdd>::from_request(request)?.0;
    let prefix = data.prefix;

    let parsed_source = data.source.parse::<Uri>()?;

    let host = parsed_source.host().map_or_else(|| Err(anyhow!("Source had no host")), |e| Ok(e))?;
    let port = parsed_source.port_u16().or(Some(80)).unwrap();
    let full_host = format!("{host}:{port:?}");
    println!("{full_host}");
    let mut stream = TcpStream::connect_timeout(full_host.clone(), Duration::from_secs(5))?;

    let path_query = parsed_source.path_and_query().map_or_else(|| Err(anyhow!("no path or query")), |m| Ok(m))?.as_str();
    stream.write_all(format!("GET {path_query} HTTP/1.0\r\n").as_bytes())?;
    stream.write_all(b"User-Agent: Custom-Lunatic/0.0\r\n")?;
    stream.write_all(b"Connection: keep-alive\r\n")?;
    stream.write_all(format!("Host: {full_host}").as_bytes())?;
    stream.write_all(b"\r\n")?;

    // no body
    stream.write_all(b"\r\n")?;
    
    stream.flush()?;
    println!("request out");
    let mut reader = BufReader::new(stream.clone());
    
    let mut start_line = String::new();
    reader.read_line(&mut start_line)?;
    println!("read {start_line}");
    if start_line != "HTTP/1.0 200 OK\r\n" {
        return Err(anyhow!("Remote returned non-ok or non-HTTP/1.0 response"));
    }

    let mut content_length : usize = 0;
    // skip all response headers, we don't care
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" {
            break
        }
        if let Some(("Content-Length:", content_length_string)) = line.split_once(':') {
            content_length = content_length_string.parse::<usize>()?;
        }
    }
    println!("read content length {content_length:?}");

    if content_length > 1024 * 1024 * 5 {
        return Err(anyhow!("No service module > 5mb allowed"))
    }

    let mut module_data = Vec::<u8>::with_capacity(content_length);
    if content_length > 0 {
        reader.read_exact(&mut module_data)?;
    }
    else {
        reader.read_to_end(&mut module_data)?;
    }
    println!("read all data");
    let len = module_data.len();

    router::add_service(prefix.clone(), module_data);

    Ok(Response::builder()
    .version(request.version())
    .status(200)
    .body(format!("OK.\n Added Service {prefix:?} from remote {full_host:?} with size: {len}").as_bytes().to_vec())?)
}

fn service_handler(request: &mut RequestContext) -> Response<Vec<u8>> {
    match request.uri().path() {
        "/services/add" => {
            service_add(request)
                .unwrap_or_else(|e| {
                    println!("{e}");
                    Response::builder()
                    .version(request.version())
                    .status(503)
                    .body(vec![]).expect("503 builder has to succeed")
                })
        }
        _ => Response::builder()
                    .version(request.version())
                    .status(404)
                    .body(vec![]).expect("404 builder has to succeed")
    }
}

impl Handler for AppHandler {
    fn handle(&self, mut context: RequestContext) -> Response<Vec<u8>> {
        // let mailbox : Mailbox<crate::http::Response> = context.mailbox;
        let mailbox : Mailbox<crate::http::Response> = unsafe { Mailbox::new() };

        let path = context.uri().path();
        let prefix = match path.split_once('/') {
            Some(("", rest)) => match rest.split_once('/') {
                Some((prefix, _rest)) => Some(prefix),
                _ => Some(rest)
            },
            Some((prefix, _rest)) => Some(prefix),
            _ => None
        };
        
        let response = match prefix {
            Some("services") => {
                service_handler(&mut context).into()
            }
            Some(prefix) => match router::create_request(prefix.to_owned()) {
                Some((service_id, request_id)) => {
                    let request = context.request;
                    let (m, b) = request.into_parts();
                    let req = frenezulo::Request
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
                        .version(context.version())
                        .status(404)
                        .body(b"Unknown Service".to_vec()).expect("404 builder has to succeed")
            },
            None => Response::builder()
                    .version(context.version())
                    .status(404)
                    .body(b"Path did not include service prefix".to_vec()).expect("404 builder has to succeed")
        };

        // for testing: restart requests after each HTTP request
        // response.headers_mut().typed_insert(submillisecond::headers::Connection::close());

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