use std::{io::{Write, Read}, error};

use bytes::{BufMut, Buf};
use submillisecond::{http::{Request, Method, Version, HeaderValue}, Body};
pub fn serialize_request<'a>(request : Request<Body<'a>>) -> Result<Vec<u8>, Box<dyn error::Error>> {
    let (parts, body) = request.into_parts();
    let header_entry_count = parts.headers.len() as u64;
    let header_bytes_count : usize =
        parts.headers
        .iter()
        .map(|(k, v)| 4 + k.as_str().as_bytes().len() + 4 + v.as_bytes().len())
        .sum::<usize>();
    let uri_string = parts.uri.to_string();
    let total_len = 1 + 1 + 4 + uri_string.as_bytes().len() + 4 + header_bytes_count + 4 + body.len();

    let mut buffer = Vec::<u8>::with_capacity(total_len);
    let mut writer = buffer.writer();
    writer.write(&[encode_method(parts.method).expect("method encoding")])?;
    writer.write(&[encode_version(parts.version).expect("version encoding")])?;
    writer.write(&(uri_string.as_bytes().len() as u64).to_le_bytes())?;
    writer.write(uri_string.as_bytes())?;
    writer.write(&(header_entry_count as u64).to_le_bytes())?;
    for (k, v) in parts.headers.iter() {
        let k_bytes = k.as_str().as_bytes();
        writer.write(&(k_bytes.len() as u64).to_le_bytes())?;
        writer.write(k_bytes)?;
        let v_bytes = v.as_bytes();
        writer.write(&(v_bytes.len() as u64).to_le_bytes())?;
        writer.write(v_bytes)?;
    }
    writer.write(&(body.len() as u64).to_le_bytes())?;
    writer.write(body.as_slice())?;
    Ok(buffer)
}

pub fn deserialize_request<'a>(data: &'a [u8]) -> Result<Request<Vec<u8>>, Box<dyn error::Error>> {
    let mut builder = Request::builder();
    let mut reader = data.reader();
    let num_bytes = [0u8; 8];
    let single_byte = [0u8; 1];
    reader.read_exact(&mut single_byte)?;
    let method = decode_method(single_byte[0]).expect("method decoding");
    builder = builder.method(method);

    reader.read_exact(&mut single_byte)?;
    let version = decode_version(single_byte[0]).expect("version decoding");
    builder = builder.version(version);
    
    reader.read_exact(&mut num_bytes)?;
    let uri_string_bytes_length = u64::from_le_bytes(num_bytes) as usize;
    let uri_string_bytes = Vec::<u8>::with_capacity(uri_string_bytes_length);
    reader.read_exact(&mut uri_string_bytes)?;
    let uri_string = String::from_utf8(uri_string_bytes)?;
    builder = builder.uri(uri_string);
    
    reader.read_exact(&mut num_bytes)?;
    let header_entry_count = u64::from_le_bytes(num_bytes) as usize;
    let &mut headers = builder.headers_mut().expect("headers");
    headers.reserve(header_entry_count - headers.capacity());
    for i in 0..header_entry_count {
        reader.read_exact(&mut num_bytes)?;
        let k_length = u64::from_le_bytes(num_bytes) as usize;

        let k_bytes = Vec::<u8>::with_capacity(k_length);
        reader.read_exact(&mut k_bytes)?;
        let k = String::from_utf8(k_bytes)?;
        
        reader.read_exact(&mut num_bytes)?;
        let v_length = u64::from_le_bytes(num_bytes) as usize;

        let v_bytes = Vec::<u8>::with_capacity(v_length);
        reader.read_exact(&mut v_bytes)?;

        headers.insert::<&str>(k.as_ref(), HeaderValue::from_maybe_shared(v_bytes)?);
    };
    reader.read_exact(&mut num_bytes)?;
    let body_length = u64::from_le_bytes(num_bytes) as usize;
    let body_buffer = Vec::<u8>::with_capacity(body_length);
    reader.read_exact(&mut body_buffer)?;
    Ok(builder.body(body_buffer)?)
}

pub fn encode_method(method : Method) -> Option<u8> {
    match method {
        Method::CONNECT => Some(0),
        Method::DELETE => Some(1),
        Method::GET => Some(2),
        Method::HEAD => Some(3),
        Method::OPTIONS => Some(4),
        Method::PATCH => Some(5),
        Method::POST => Some(6),
        Method::PUT => Some(7),
        Method::TRACE => Some(8),
        _ => None
    }
}

pub fn decode_method(input: u8) -> Option<Method> {
    match input {
        0 => Some(Method::CONNECT),
        1 => Some(Method::DELETE),
        2 => Some(Method::GET),
        3 => Some(Method::HEAD),
        4 => Some(Method::OPTIONS),
        5 => Some(Method::PATCH),
        6 => Some(Method::POST),
        7 => Some(Method::PUT),
        8 => Some(Method::TRACE),
        _ => None
    }
}

pub fn encode_version(version : Version) -> Option<u8> {
    match version {
        Version::HTTP_10 => Some(1),
        Version::HTTP_11 => Some(2),
        Version::HTTP_2 => Some(3),
        Version::HTTP_3 => Some(4),
        _ => None
    }
}

pub fn decode_version(input: u8) -> Option<Version> {
    match input {
        1 => Some(Version::HTTP_10),
        2 => Some(Version::HTTP_11),
        3 => Some(Version::HTTP_2),
        4 => Some(Version::HTTP_3),
        _ => None
    }
}