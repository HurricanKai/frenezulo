use submillisecond::{router, Application, http::HeaderMap};

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
}

fn main() -> std::io::Result<()> {
    Application::new(router! {
        GET "/" => index
        GET "/headers" => headers
    })
    .serve("0.0.0.0:3000")
}