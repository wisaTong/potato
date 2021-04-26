use crate::prep;
use crate::request::{HttpRequestMethod, PotatoRequest};
use crate::response::PotatoResponse;
use libpotato::{clone, idmap, libc, net, nix, signal, signal_hook as sighook};
use nix::unistd;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

pub type PotatoRequestHandler = fn(PotatoRequest) -> PotatoResponse;
#[derive(Eq, PartialEq, Hash)]
pub struct PotatoRoute<'a> {
    method: HttpRequestMethod,
    path: &'a str,
}

pub struct PotatoServer<'a> {
    port: String,
    runtime_dir: String,
    handlers: HashMap<PotatoRoute<'a>, PotatoRequestHandler>,
    default_handler: Option<PotatoRequestHandler>,
    isolation: bool,
}

impl<'a> PotatoServer<'a> {
    pub fn new(port: &str, runtime_dir: &str, isolation: bool) -> PotatoServer<'a> {
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
        path: &'a str,
        handler: PotatoRequestHandler,
    ) -> PotatoServer<'a> {
        let route = PotatoRoute { method, path };
        self.handlers.insert(route, handler);
        self
    }

    pub fn add_default_handler(mut self, handler: PotatoRequestHandler) -> PotatoServer<'a> {
        self.default_handler = Some(handler);
        self
    }

    pub fn start(self) {
        let address = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&address).unwrap();

        // Create runtime directory
        fs::create_dir_all(&self.runtime_dir).expect("Failed to initialized runtime directoy");

        if self.isolation {
            // FIXME preparing bridge in the host probably not require in code.
            // because we want to be able to run web server without root permission
            net::prep_bridge("10.0.0.0/24".to_string());
            // Install signal handler for reaping child
            signal::install_sigchld_handler().expect("Failed installing SIGCHLD handler");
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
            if self.isolation {
                self.handle_connection_with_isolation(stream);
            } else {
                self.handle_connection(stream)
            }
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
                let req = PotatoRequest::new(route.method, route.path);
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
                let req = PotatoRequest::new(route.method, route.path);
                self.isolate_req(stream, req, *handler);
                break;
            }
        }
    }

    fn isolate_req(&self, stream: TcpStream, req: PotatoRequest, handler: PotatoRequestHandler) {
        const STACK_SIZE: usize = 1024 * 1024;
        let ref mut child_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let ref mut init_stack: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let copy_stream = stream.try_clone().unwrap();
        let rootfs = prep::fs_prep(&self.runtime_dir);

        let flags = libc::CLONE_NEWUSER
            | libc::CLONE_NEWPID
            | libc::CLONE_NEWNS
            | libc::CLONE_NEWNET
            | libc::CLONE_NEWCGROUP
            | libc::CLONE_NEWUTS
            | libc::CLONE_NEWIPC
            | libc::SIGCHLD;

        let cb = |s| {
            || {
                // start in suspended state
                unsafe { libc::raise(libc::SIGSTOP) };
                unistd::chroot(rootfs.as_str()).unwrap();
                unistd::chdir(".").unwrap();
                let res = handler(req);
                self.write_response(s, res);
                0
            }
        };

        let init_cb = || {
            let copy_stream = stream.try_clone().unwrap();
            match clone::clone_proc_newns(cb(stream), child_stack, libc::SIGCHLD) {
                Ok(pid) => {
                    let sigs = [libc::SIGCONT, libc::SIGCHLD];
                    let mut siginfo = sighook::iterator::Signals::new(&sigs).unwrap();
                    signal::unblock(&[nix::sys::signal::SIGCONT]).unwrap();
                    for sig in siginfo.forever() {
                        match sig {
                            libc::SIGCONT => unsafe {
                                libc::kill(pid, sig);
                            },
                            libc::SIGCHLD => {
                                fs::remove_dir_all(&rootfs).unwrap();
                                std::process::exit(0);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Err(_) => {
                    self.handle_req_error(copy_stream, "Isolation failure: clone cb");
                    return -1;
                }
            }

            0
        };

        signal::block(&[nix::sys::signal::SIGCONT]).unwrap();
        match clone::clone_proc_newns(init_cb, init_stack, flags) {
            Ok(pid) => {
                signal::unblock(&[nix::sys::signal::SIGCONT]).unwrap();
                // BUNCHA SETUP
                idmap::UidMapper::new()
                    .add(0, unistd::getuid().as_raw(), 1)
                    .write_newuidmap(pid)
                    .unwrap();

                // send SIGCONT after finished setup
                unsafe { libc::kill(pid, libc::SIGCONT) };
            }
            Err(_) => self.handle_req_error(copy_stream, "Isolation failure: clone init"),
        }
    }

    fn write_response(&self, mut stream: TcpStream, response: PotatoResponse) {
        let res = response.to_http_response();
        stream.write(&res).unwrap();
        stream.flush().unwrap();
    }

    fn handle_req_error(&self, mut stream: TcpStream, message: &str) {
        let header = format!("Content-Length: {}", message.len());
        let response = format!(
            "HTTP/1.1 500 Internal Server Error\r\n{}\r\n\r\n{}",
            header, message
        );
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}
