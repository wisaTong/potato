use std::collections::HashMap;
use std::fmt;

#[allow(dead_code)]
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub enum HttpRequestMethod {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl HttpRequestMethod {
    fn to_string(&self) -> String {
        let name = match self {
            HttpRequestMethod::GET => "GET",
            HttpRequestMethod::HEAD => "HEAD",
            HttpRequestMethod::POST => "POST",
            HttpRequestMethod::PUT => "PUT",
            HttpRequestMethod::DELETE => "DELETE",
            HttpRequestMethod::CONNECT => "CONNECT",
            HttpRequestMethod::OPTIONS => "OPTIONS",
            HttpRequestMethod::TRACE => "TRACE",
            HttpRequestMethod::PATCH => "PATCH",
        };
        name.to_string()
    }
}

impl fmt::Display for HttpRequestMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct PotatoRequest {
    pub method: HttpRequestMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl PotatoRequest {
    pub fn new(method: HttpRequestMethod, path: &str) -> PotatoRequest {
        PotatoRequest {
            method,
            path: path.to_string(),
            headers: HashMap::new(),
            body: None,
        }
    }
}
