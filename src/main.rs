use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use nix::sched::*;
use nix::unistd;
// --snip--

struct PotatoResponse<'a> {
    status_code: &'a str,
    header: &'a str,
    body: &'a str
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn get_hostname_as_string() ->  String {
    let mut buf = [0u8; 64];
    let hostname_cstr = unistd::gethostname(&mut buf).expect("Failed getting hostname");
    let hostname = hostname_cstr.to_str().expect("Hostname wasn't valid UTF-8");
    hostname.to_string()
}

fn get_hostname<'a>() -> PotatoResponse<'a> {
    let status_code = "200 Ok";
    let header = "";
    let body = get_hostname_as_string();
    let response = PotatoResponse{status_code, header, body};
    response
}

fn set_and_get_hostname<'a>(new_host: String) -> PotatoResponse<'a> {
    unistd::sethostname(new_host);
    let status_code = "200 Ok";
    let header = "";
    let body = get_hostname_as_string();
    let response = PotatoResponse{status_code, header, body};
    response
}

fn write_response<'a, F>(logic: F, mut stream: &TcpStream) -> isize 
where 
    F: FnOnce() -> PotatoResponse<'a>,
{
    let response = logic();
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    0
}

fn handle_connection(mut stream: TcpStream) {
    // --snip--
    let get = b"GET / HTTP/1.1\r\n";
    let get_ns = b"GET /ns HTTP/1.1\r\n";
    
    const STACK_SIZE: usize = 1024*1024;
    let ref mut stack: [u8; STACK_SIZE] = [0; STACK_SIZE];
    
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    
    if buffer.starts_with(get) {
        let cb = Box::new(|| write_response(get_hostname(), &stream));
        clone(cb, stack, CloneFlags::CLONE_NEWUTS, None);
    } else if buffer.starts_with(get_ns) {
        let cb = Box::new(|| write_response(set_and_get_hostname("booknap"), &stream));
        clone(cb, stack, CloneFlags::CLONE_NEWUTS, None);
    }
}
