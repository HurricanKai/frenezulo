use multimap::MultiMap;
use serde::{Serialize, Deserialize};
use submillisecond::http::StatusCode;


#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub metadata: RequestMetadata,
    pub body: serde_bytes::ByteBuf,
}

impl std::convert::From<submillisecond::http::Request<Vec<u8>>> for Request {
    fn from(source: submillisecond::http::Request<Vec<u8>>) -> Self {
        let (parts, body) = source.into_parts();
        Self { metadata: RequestMetadata::from(parts), body: serde_bytes::ByteBuf::from(body) }
    }
}

impl std::convert::From<Request> for submillisecond::http::Request<Vec<u8>> {
    fn from(source: Request) -> Self {
        Self::from_parts(source.metadata.into(), source.body.into_vec())
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub metadata: ResponseMetadata,
    pub body: serde_bytes::ByteBuf,
}

impl std::convert::From<submillisecond::http::Response<Vec<u8>>> for Response {
    fn from(source: submillisecond::http::Response<Vec<u8>>) -> Self {
        let (parts, body) = source.into_parts();
        Self { metadata: ResponseMetadata::from(parts), body: serde_bytes::ByteBuf::from(body) }
    }
}

impl std::convert::From<Response> for submillisecond::http::Response<Vec<u8>> {
    fn from(source: Response) -> Self {
        Self::from_parts(source.metadata.into(), source.body.into_vec())
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestMetadata {    
    pub method: Method,
    pub uri: String,
    pub version: Version,
    pub headers: MultiMap<String, serde_bytes::ByteBuf>,
}

impl std::convert::From<submillisecond::http::request::Parts> for RequestMetadata {
    fn from(parts: submillisecond::http::request::Parts) -> Self {
        Self { 
            method: Method::from(parts.method),
            uri: parts.uri.to_string(),
            version: Version::from(parts.version),
            headers: parts.headers.iter()
                .map(|(k, v)| {
                    (k.as_str().to_owned(), v.as_bytes().to_owned())
                })
                .fold(MultiMap::<String, serde_bytes::ByteBuf>::new(), |mut m, (k, v)| {
                    m.insert(k, serde_bytes::ByteBuf::from(v));
                    m
                })
        }
    }
}

impl std::convert::From<RequestMetadata> for submillisecond::http::request::Parts {
    fn from(parts: RequestMetadata) -> Self {
        Self { 
            method: parts.method.into(),
            uri: parts.uri.parse::<submillisecond::http::Uri>().expect("Parsing has to succeed"),
            version: parts.version.into(),
            headers: parts.headers.iter()
                .map(|(k, v)| {
                    (submillisecond::headers::HeaderName::from_static(k),
                    submillisecond::headers::HeaderValue::from_maybe_shared(v).expect("Header value has to be valid"))
                })
                .fold(submillisecond::headers::HeaderMap::new(), |mut m, (k, v)| {
                    m.insert(k, v);
                    m
                }),
            extensions: Default::default(),
            _priv: ()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub status: u16,
    pub version: Version,
    pub headers: MultiMap<String, serde_bytes::ByteBuf>,
}

impl std::convert::From<submillisecond::http::response::Parts> for ResponseMetadata {
    fn from(parts: submillisecond::http::response::Parts) -> Self {
        Self { 
            status: parts.status.as_u16(),
            version: Version::from(parts.version),
            headers: parts.headers.iter()
                .map(|(k, v)| {
                    (k.as_str().to_owned(), v.as_bytes().to_owned())
                })
                .fold(MultiMap::<String, serde_bytes::ByteBuf>::new(), |mut m, (k, v)| {
                    m.insert(k, serde_bytes::ByteBuf::from(v));
                    m
                })
        }
    }
}

impl std::convert::From<ResponseMetadata> for submillisecond::http::response::Parts {
    fn from(parts: ResponseMetadata) -> Self {
        Self { 
            status: StatusCode::from_u16(parts.status).expect("has to be valid"),
            version: parts.version.into(),
            headers: parts.headers.iter()
                .map(|(k, v)| {
                    (submillisecond::headers::HeaderName::from_static(k),
                    submillisecond::headers::HeaderValue::from_maybe_shared(v).expect("Header value has to be valid"))
                })
                .fold(submillisecond::headers::HeaderMap::new(), |mut m, (k, v)| {
                    m.insert(k, v);
                    m
                }),
            extensions: Default::default(),
            _priv: ()
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Method {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
}

impl std::convert::From<submillisecond::http::Method> for Method {
    fn from(source: submillisecond::http::Method) -> Self {
        match source {
            submillisecond::http::Method::OPTIONS => Method::Options,
            submillisecond::http::Method::GET => Method::Get,
            submillisecond::http::Method::POST => Method::Post,
            submillisecond::http::Method::PUT => Method::Put,
            submillisecond::http::Method::DELETE => Method::Delete,
            submillisecond::http::Method::HEAD => Method::Head,
            submillisecond::http::Method::TRACE => Method::Trace,
            submillisecond::http::Method::CONNECT => Method::Connect,
            submillisecond::http::Method::PATCH => Method::Patch,
            _ => panic!("Invalid HTTP Method")
        }
    }
}

impl std::convert::From<Method> for submillisecond::http::Method {
    fn from(source: Method) -> Self {
        match source {
            Method::Options => submillisecond::http::Method::OPTIONS,
            Method::Get => submillisecond::http::Method::GET,
            Method::Post => submillisecond::http::Method::POST,
            Method::Put => submillisecond::http::Method::PUT,
            Method::Delete => submillisecond::http::Method::DELETE,
            Method::Head => submillisecond::http::Method::HEAD,
            Method::Trace => submillisecond::http::Method::TRACE,
            Method::Connect => submillisecond::http::Method::CONNECT,
            Method::Patch => submillisecond::http::Method::PATCH,
            _ => panic!("Invalid HTTP Method")
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Version {
    Http09,
    Http10,
    Http11,
    Http2,
    Http3,
}

impl std::convert::From<submillisecond::http::Version> for Version {
    fn from(source: submillisecond::http::Version) -> Self {
        match source {
            submillisecond::http::Version::HTTP_09 => Version::Http09,
            submillisecond::http::Version::HTTP_10 => Version::Http10,
            submillisecond::http::Version::HTTP_11 => Version::Http11,
            submillisecond::http::Version::HTTP_2 => Version::Http2,
            submillisecond::http::Version::HTTP_3 => Version::Http3,
            _ => panic!("Invalid HTTP Version")
        }
    }
}

impl std::convert::From<Version> for submillisecond::http::Version {
    fn from(source: Version) -> Self {
        match source {
            Version::Http09 => submillisecond::http::Version::HTTP_09,
            Version::Http10 => submillisecond::http::Version::HTTP_10,
            Version::Http11 => submillisecond::http::Version::HTTP_11,
            Version::Http2 => submillisecond::http::Version::HTTP_2,
            Version::Http3 => submillisecond::http::Version::HTTP_3,
            _ => panic!("Invalid HTTP Version")
        }
    }
}