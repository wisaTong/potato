use crate::request::{
    HttpRequestMethod::{self, *},
    PotatoRequest,
};
use crate::response::PotatoResponse;
use crate::{isolation, prep};
use libpotato::{net, signal};
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
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
            signal::ignore_sigchld().expect("Abort: cuz dont want zombie");
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

        let protected_runtime_dir = Arc::new(Mutex::new(self.runtime_dir.clone()));

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let server = self.clone();

            let arc_runtime_dir = Arc::clone(&protected_runtime_dir);
            thread::spawn(move || {
                if server.isolation {
                    let runtime_dir = arc_runtime_dir.lock().unwrap();
                    let rootfs = prep::fs_prep(&runtime_dir);
                    std::mem::drop(runtime_dir); // unlock mutex
                    server.handle_connection_with_isolation(stream, rootfs);
                } else {
                    server.handle_connection(stream);
                }
            });
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let ref mut buffer: [u8; 1024] = [0; 1024];
        stream.read(buffer).unwrap();

        let len = &self.handlers.len();
        let mut count: usize = 1;

        let req = PotatoRequest::from_raw_req(buffer);

        for (route, handler) in &self.handlers {
            let head = format!("{} {} HTTP/1.1", route.method, route.path);
            if buffer.starts_with(head.as_bytes()) {
                let pres = handler(req);
                self.write_response(stream, pres);
                break;
            } else if len.eq(&count) {
                let d_handler = self.default_handler.unwrap();
                let pres = d_handler(req);
                self.write_response(stream, pres);
                break;
            }
            count += 1;
        }
    }

    fn handle_connection_with_isolation(&self, mut stream: TcpStream, rootfs: String) {
        let ref mut buffer: [u8; 1024] = [0; 1024];
        stream.read(buffer).unwrap();

        let len = &self.handlers.len();
        let mut count: usize = 1;

        let req = PotatoRequest::from_raw_req(buffer);

        for (route, handler) in &self.handlers {
            let head = format!("{} {} HTTP/1.1", route.method, route.path);
            if buffer.starts_with(head.as_bytes()) {
                let req = PotatoRequest::from_raw_req(buffer);
                if let Err(strm) = isolation::isolate_req(stream, req, *handler, &rootfs) {
                    self.handle_req_error(&strm, "Isolation failure: clone init");
                }
                break;
            } else if len.eq(&count) {
                let d_handler = self.default_handler.unwrap();
                if let Err(strm) = isolation::isolate_req(stream, req, d_handler, &rootfs) {
                    self.handle_req_error(&strm, "Isolation failure: clone init");
                }
                break;
            }
            count += 1;
        }
    }

    fn write_response(&self, mut stream: TcpStream, response: PotatoResponse) {
        let res = response.to_http_response();
        stream.write(&res).unwrap();
        stream.flush().unwrap();
    }

    pub fn get_header(&self, s: &str, ignore: &str) -> HashMap<String, String> {
        let mut check: bool = true;
        let mut header: HashMap<String, String> = HashMap::new();
        for x in s.lines() {
            if x.contains(ignore) {
                continue;
            }
            if x.starts_with("-") {
                check = false;
            }
            if check {
                if x.is_empty() {
                    break;
                }
                let (a, b) = x.split_at(x.find(":").unwrap());
                header.insert(a.to_string(), b.to_string());
            }
        }
        header
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
