use crate::request::{HttpRequestMethod, PotatoRequest};
use crate::response::PotatoResponse;
use libpotato::signal;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

pub type PotatoRequestHandler<'a> = fn(PotatoRequest) -> PotatoResponse<'a>;

#[derive(Eq, PartialEq, Hash)]
pub struct PotatoRoute<'a> {
    method: HttpRequestMethod,
    path: &'a str,
}

pub struct PotatoServer<'a> {
    port: String,
    runtime_dir: String,
    handlers: HashMap<PotatoRoute<'a>, PotatoRequestHandler<'a>>,
}

impl<'a> PotatoServer<'a> {
    pub fn new(port: &str, runtime_dir: &str) -> PotatoServer<'a> {
        PotatoServer {
            port: port.to_string(),
            runtime_dir: runtime_dir.to_string(),
            handlers: HashMap::new(),
        }
    }

    pub fn add_handler(
        mut self,
        method: HttpRequestMethod,
        path: &'a str,
        handler: PotatoRequestHandler<'a>,
    ) -> PotatoServer<'a> {
        let route = PotatoRoute { method, path };
        self.handlers.insert(route, handler);
        self
    }

    pub fn start(self) {
        let address = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&address).unwrap();

        // Create runtime directory
        fs::create_dir_all(&self.runtime_dir).expect("Failed to initialized runtime directoy");

        // Install signal handler for reaping child
        signal::install_sigchld_handler().expect("Failed installing SIGCHLD handler");

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
            self.handle_connection(stream);
        }
    }

    fn handle_connection(&self, mut stream: TcpStream) {
        let ref mut buffer: [u8; 1024] = [0; 1024];
        stream.read(buffer).unwrap();

        for (route, handler) in &self.handlers {
            let head = format!("{} {} HTTP/1.1", route.method, route.path);
            if buffer.starts_with(head.as_bytes()) {
                let req = PotatoRequest::new(route.method, route.path);
                let pres = handler(req);
                self.write_response(stream, pres);
                break;
            }
        }
    }

    fn write_response(&self, mut stream: TcpStream, response: PotatoResponse) {
        let res = response.to_http_response();
        stream.write(&res).unwrap();
        stream.flush().unwrap();
    }
}