use std::collections::HashMap;

pub struct PotatoResponse<'a> {
    status: String,
    headers: HashMap<String, String>,
    body: Option<&'a [u8]>,
}

impl<'a> PotatoResponse<'a> {
    pub fn new() -> PotatoResponse<'a> {
        PotatoResponse {
            status: String::new(),
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn set_status(mut self, status: &str) -> PotatoResponse<'a> {
        self.status = status.to_string();
        self
    }

    pub fn add_header(mut self, key: &str, value: &str) -> PotatoResponse<'a> {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn add_body(mut self, content: &'a [u8]) -> PotatoResponse<'a> {
        self.body = Some(content);
        self
    }

    pub fn to_http_response(&self) -> Vec<u8> {
        let mut headers = String::new();
        for (k, v) in &self.headers {
            headers = format!("{}: {}\n", k, v);
        }

        let response = format!("HTTP/1.1 {}\r\n{}\r\n\r\n", self.status, headers.trim_end());
        if let Some(body) = self.body {
            return [response.as_bytes(), body].concat();
        }

        [response.as_bytes()].concat()
    }
}
