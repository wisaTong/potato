use std::collections::HashMap;

pub struct PotatoResponse {
    status: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
}

impl PotatoResponse {
    pub fn new() -> PotatoResponse {
        PotatoResponse {
            status: String::new(),
            headers: HashMap::new(),
            body: None,
        }
    }

    pub fn set_status(mut self, status: &str) -> PotatoResponse {
        self.status = status.to_string();
        self
    }

    pub fn add_header(mut self, key: &str, value: &str) -> PotatoResponse {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn add_body(mut self, content: Vec<u8>) -> PotatoResponse {
        self.body = Some(content);
        self
    }

    pub fn to_http_response(&self) -> Vec<u8> {
        let mut headers = String::new();
        for (k, v) in &self.headers {
            headers = format!("{}: {}\n", k, v);
        }

        let response = format!("HTTP/1.1 {}\r\n{}\r\n\r\n", self.status, headers.trim_end());
        if let Some(body) = &self.body {
            return [response.as_bytes(), &body].concat();
        }

        [response.as_bytes()].concat()
    }
}
