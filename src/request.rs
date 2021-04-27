use std::collections::HashMap;
use std::fmt;

use libpotato::libc::printf;

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

    fn from_str(s: &str) -> Option<HttpRequestMethod> {
        match s {
            "GET" => Some(HttpRequestMethod::GET),
            "HEAD" => Some(HttpRequestMethod::HEAD),
            "POST" => Some(HttpRequestMethod::POST),
            "PUT" => Some(HttpRequestMethod::PUT),
            "DELETE" => Some(HttpRequestMethod::DELETE),
            "CONNECT" => Some(HttpRequestMethod::CONNECT),
            "OPTIONS" => Some(HttpRequestMethod::OPTIONS),
            "TRACE" => Some(HttpRequestMethod::TRACE),
            "PATCH" => Some(HttpRequestMethod::PATCH),
            _ => None,
        }
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
    pub body: Option<Vec<u8>>,
}

impl PotatoRequest {
    pub fn new(method: HttpRequestMethod, path: &str, body: Option<Vec<u8>>) -> PotatoRequest {
        PotatoRequest {
            method,
            path: path.to_string(),
            headers: HashMap::new(),
            body: body,
        }
    }

    pub fn from_raw_req(raw: &[u8]) -> PotatoRequest {
        let index = raw
            .windows(2)
            .enumerate()
            .find(|(_, w)| matches!(*w, b"\r\n"))
            .map(|(i, _)| i)
            .unwrap();

        let head = String::from_utf8_lossy(&raw[..index]);
        let (raw_method, back) = head.split_at(head.find("/").unwrap());
        let (raw_path, _) = back.split_at(back.find("H").unwrap());

        let method = match HttpRequestMethod::from_str(raw_method.trim()){
            std::option::Option::Some(method) => {method}
            std::option::Option::None => {HttpRequestMethod::OPTIONS}
        };
        let path = raw_path.trim();

        let bindex = raw
            .windows(4)
            .enumerate()
            .find(|(_, w)| matches!(*w, b"\r\n\r\n"))
            .map(|(i, _)| i)
            .unwrap();

        let headers = String::from_utf8_lossy(&raw[index..bindex]);
        let mut content_len = 0;
        for i in headers.lines() {
            if i.is_empty() {
                continue;
            }
            let (a, b) = i.split_at(i.find(":").unwrap());
            if a.trim().starts_with("Content-Length") {
                content_len = b.replace(": ", "").to_string().parse::<usize>().unwrap();
            }
        }
        let body = raw[bindex + 4..content_len + bindex + 4].to_vec();

        PotatoRequest::new(method, path, Some(body))
    }
}
