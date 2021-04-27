use crate::request::{HttpRequestMethod, PotatoRequest};
use crate::response::PotatoResponse;
use crate::{
    isolation::{isolate_req, IsolationSetting},
    prep,
};
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
    handlers: Vec<(PotatoRoute, PotatoRequestHandler, Option<IsolationSetting>)>,
    default_handler: Option<(PotatoRequestHandler, Option<IsolationSetting>)>,
    isolation: bool,
}

impl PotatoServer {
    pub fn new(port: &str, runtime_dir: &str, isolation: bool) -> PotatoServer {
        PotatoServer {
            port: port.to_string(),
            runtime_dir: runtime_dir.to_string(),
            handlers: Vec::new(),
            default_handler: None,
            isolation,
        }
    }

    pub fn add_handler(
        self,
        method: HttpRequestMethod,
        path: &str,
        handler: PotatoRequestHandler,
    ) -> PotatoServer {
        self.add_handler_with_isolation(method, path, handler, None)
    }

    pub fn add_handler_with_isolation(
        mut self,
        method: HttpRequestMethod,
        path: &str,
        handler: PotatoRequestHandler,
        opt_isolation: Option<IsolationSetting>,
    ) -> PotatoServer {
        let route = PotatoRoute {
            method,
            path: path.to_string(),
        };
        self.handlers.push((route, handler, opt_isolation));
        self
    }

    pub fn add_default_handler(self, handler: PotatoRequestHandler) -> PotatoServer {
        self.add_default_handler_with_isolation(handler, None)
    }

    pub fn add_default_handler_with_isolation(
        mut self,
        handler: PotatoRequestHandler,
        opt_isolation: Option<IsolationSetting>,
    ) -> PotatoServer {
        self.default_handler = Some((handler, opt_isolation));
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
            let server = self.clone();
            let stream = stream.unwrap();
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
        let ref mut buffer = [0; 1024];
        if let Ok(0) = stream.read(buffer) {
            self.handle_req_error(&stream, "Socket closed");
            return;
        }

        let len = &self.handlers.len();
        let mut count: usize = 1;

        let req = PotatoRequest::from_raw_req(buffer);

        for (route, handler, _) in &self.handlers {
            let head = format!("{} {} HTTP/1.1", route.method, route.path);
            if buffer.starts_with(head.as_bytes()) {
                let pres = handler(req);
                self.write_response(stream, pres);
                break;
            } else if len.eq(&count) {
                let (d_handler, _) = self.default_handler.clone().unwrap();
                let pres = d_handler(req);
                self.write_response(stream, pres);
                break;
            }
            count += 1;
        }
    }

    fn handle_connection_with_isolation(&self, mut stream: TcpStream, rootfs: String) {
        let ref mut buffer = [0; 1024];
        if let Ok(0) = stream.read(buffer) {
            self.handle_req_error(&stream, "Socket closed");
            return;
        }

        let len = &self.handlers.len();
        let mut count: usize = 1;

        let req = PotatoRequest::from_raw_req(buffer);
        for (route, handler, opt_isolation) in &self.handlers {
            let head = format!("{} {} HTTP/1.1", route.method, route.path);

            if buffer.starts_with(head.as_bytes()) {
                let mut isolation_setting = opt_isolation.clone().unwrap(); // safe unwrap
                isolation_setting.rootfs_path = rootfs;

                if let Err(strm) = isolate_req(stream, req, *handler, isolation_setting) {
                    self.handle_req_error(&strm, "Isolation failure: clone init");
                }
                break;
            } else if len.eq(&count) {
                let (d_handler, opt_isolation) = self.default_handler.clone().unwrap();
                let mut isolation_setting = opt_isolation.clone().unwrap(); // safe unwrap
                isolation_setting.rootfs_path = rootfs;
                if let Err(strm) = isolate_req(stream, req, d_handler, isolation_setting) {
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
