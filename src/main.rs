use libc;
use nix::unistd;
use potato::clone;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

struct PotatoResponse {
    status_code: String,
    header: String,
    body: String,
}

impl PotatoResponse {
    fn to_http_response(&self) -> String {
        format!(
            "HTTP/1.1 {}\r\n{}\r\n\r\n{}",
            self.status_code, self.header, self.body
        )
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn get_hostname_as_string() -> String {
    let mut buf = [0u8; 64];
    let hostname_cstr = unistd::gethostname(&mut buf).expect("Failed getting hostname");
    let hostname = hostname_cstr.to_str().expect("Hostname wasn't valid UTF-8");
    hostname.to_string()
}

fn get_hostname() -> PotatoResponse {
    let status_code = "200 Ok".to_string();
    let header = "".to_string();
    let body = get_hostname_as_string();
    let response = PotatoResponse {
        status_code,
        header,
        body,
    };
    response
}

fn set_and_get_hostname() -> PotatoResponse {
    unistd::sethostname("boba").unwrap();
    PotatoResponse {
        status_code: "200 OK".to_string(),
        header: "".to_string(),
        body: get_hostname_as_string(),
    }
}

fn write_response<F>(logic: F, mut stream: &TcpStream) -> isize
where
    F: FnOnce() -> PotatoResponse,
{
    let response = logic().to_http_response();
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    0
}

fn handle_connection(mut stream: TcpStream) {
    let get = b"GET / HTTP/1.1\r\n";
    let get_ns = b"GET /ns HTTP/1.1\r\n";

    const STACK_SIZE: usize = 1024 * 1024;
    let ref mut stack: [u8; STACK_SIZE] = [0; STACK_SIZE];

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    if buffer.starts_with(get) {
        let cb = || write_response(get_hostname, &stream);
        clone::clone_proc_newns(cb, stack, libc::CLONE_NEWUTS | libc::CLONE_NEWNET);
    }
    if buffer.starts_with(get_ns) {
        let cb = || write_response(set_and_get_hostname, &stream);
        clone::clone_proc_newns(
            cb,
            stack,
            libc::CLONE_NEWUTS
                | libc::CLONE_NEWNET
                | libc::CLONE_VM
                | libc::CLONE_THREAD
                | libc::CLONE_SIGHAND,
        );
    }
}
