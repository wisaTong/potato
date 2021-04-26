use crate::isolation;
use crate::request::{HttpRequestMethod, PotatoRequest};
use crate::response::PotatoResponse;
use libpotato::{net, signal};
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::thread;

pub type PotatoRequestHandler = fn(PotatoRequest) -> PotatoResponse;

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct PotatoRoute {
    method: HttpRequestMethod,
    path: String,
}

#[derive(Clone)]
pub struct PotatoServer {
    port: String,
    runtime_dir: String,
    handlers: HashMap<PotatoRoute, PotatoRequestHandler>,
    default_handler: Option<PotatoRequestHandler>,
    isolation: bool,
}

impl PotatoServer {
    pub fn new(port: &str, runtime_dir: &str, isolation: bool) -> PotatoServer {
        PotatoServer {
            port: port.to_string(),
            runtime_dir: runtime_dir.to_string(),
            handlers: HashMap::new(),
            default_handler: None,
            isolation,
        }
    }

    pub fn add_handler(
        mut self,
        method: HttpRequestMethod,
        path: &str,
        handler: PotatoRequestHandler,
    ) -> PotatoServer {
        let route = PotatoRoute {
            method,
            path: path.to_string(),
        };
        self.handlers.insert(route, handler);
        self
    }

    pub fn add_default_handler(mut self, handler: PotatoRequestHandler) -> PotatoServer {
        self.default_handler = Some(handler);
        self
    }

    pub fn start(self) {
        let address = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&address).unwrap();

        // Create runtime directory
        fs::create_dir_all(&self.runtime_dir).expect("Failed to initialized runtime directoy");

        if self.isolation {
            // FIXME preparing bridge in the host probably not require in code.
            // because we want to be able to run web server without root permission
            net::prep_bridge("10.0.0.0/24".to_string());
            signal::install_sigchld_sigign().expect("Failed to install SIGCHLD handler, zombiee");
        }

        let startup_message = format!(
            "\n        ▒▒▒▒▒▒▒▒▓▓                                                                           \
             \n    ▒▒▒▒░░░░░░██░░▓▓▓▓                                  ,d                 ,d                \
             \n    ▒▒░░░░░░██▓▓░░░░▒▒▓▓                                88                 88                \
             \n  ▒▒░░░░▓▓░░░░░░░░░░▒▒▓▓▓▓     8b,dPPYba,   ,adPPYba, MM88MMM ,adPPYYba, MM88MMM ,adPPYba,   \
             \n  ▒▒░░░░░░░░░░░░░░░░▒▒▓▓▓▓     88P'    '8a a8'     '8a  88    **     `Y8   88   a8'     '8a  \
             \n  ▒▒░░░░░░░░░░██░░▒▒▓▓▓▓▓▓     88       d8 8b       d8  88    ,adPPPPP88   88   8b       d8  \
             \n  ▒▒░░░░░░░░▓▓██▒▒▓▓▓▓▓▓       88b,   ,a8' '8a,   ,a8'  88,   88,    ,88   88,  '8a,   ,a8'  \
             \n  ▓▓░░░░░░░░░░▒▒▓▓▓▓▓▓         88`YbbdP''   `'YbbdP''   'Y888 `'8bbdP'Y8   'Y888 `'YbbdP''   \
             \n    ▓▓▓▓▒▒▒▒▓▓▓▓▓▓             88                                                            \
             \n        ▓▓▓▓▓▓▓▓               88    Listening on {}",
            address
        );
        println!("{}", startup_message);

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let server = self.clone();
            thread::spawn(move || {
                if server.isolation {
                    server.handle_connection_with_isolation(stream);
                } else {
                    server.handle_connection(stream);
                }
            });
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let ref mut buffer: [u8; 1024] = [0; 1024];
        stream.read(buffer).unwrap();

        let s = std::str::from_utf8(buffer).expect("can't covert utf8 to str");
        let (_, back) = s.split_at(s.find("/").unwrap() + 1);
        let (result, _) = back.split_at(s.find("H").unwrap() - 5);

        let len = &self.handlers.len();
        let mut count: usize = 1;

        for (route, handler) in &self.handlers {
            let head = format!("{} {} HTTP/1.1", route.method, route.path);
            if buffer.starts_with(head.as_bytes()) {
                let req = PotatoRequest::new(route.method, &route.path);
                let pres = handler(req);
                self.write_response(stream, pres);
                break;
            } else if len.eq(&count) {
                let req = PotatoRequest::new(HttpRequestMethod::GET, result.trim());
                let d_handler = self.default_handler.unwrap();
                let pres = d_handler(req);
                self.write_response(stream, pres);
                break;
            }
            count += 1;
        }
    }

    fn handle_connection_with_isolation(&self, mut stream: TcpStream) {
        let ref mut buffer: [u8; 1024] = [0; 1024];
        stream.read(buffer).unwrap();

        for (route, handler) in &self.handlers {
            let head = format!("{} {} HTTP/1.1", route.method, route.path);
            if buffer.starts_with(head.as_bytes()) {
                let req = PotatoRequest::new(route.method, &route.path);
                if let Err(strm) = isolation::isolate_req(stream, req, *handler) {
                    self.handle_req_error(&strm, "Isolation failure: clone init");
                }
                break;
            }
        }
    }

    fn write_response(&self, mut stream: TcpStream, response: PotatoResponse) {
        let res = response.to_http_response();
        stream.write(&res).unwrap();
        stream.flush().unwrap();
    }

    fn handle_req_error(&self, mut stream: &TcpStream, message: &str) {
        let header = format!("Content-Length: {}", message.len());
        let response = format!(
            "HTTP/1.1 500 Internal Server Error\r\n{}\r\n\r\n{}",
            header, message
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
