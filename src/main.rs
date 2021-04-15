use libc;
use nix::unistd;
use potato::clone;
use potato::net;
use std::fs;
use std::io::{self, prelude::*};
use std::net::TcpListener;
use std::net::TcpStream;
use std::process::Command;

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
    // create potato dir for each request if not exist
    let uid = unistd::getuid();
    let runtime_dir = format!("/var/run/user/{}/potato", uid);
    fs::create_dir_all(runtime_dir).expect("Faild to create runtime dir");

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

fn get_hostname(_: &TcpStream) -> PotatoResponse {
    let status_code = "200 Ok".to_string();
    let body = get_hostname_as_string();
    let header = format!("Content-Length: {}", body.len());
    PotatoResponse {
        status_code,
        header,
        body,
    }
}

fn set_and_get_hostname(_: &TcpStream) -> PotatoResponse {
    unistd::sethostname("boba").unwrap();
    let body = get_hostname_as_string();

    PotatoResponse {
        status_code: "200 OK".to_string(),
        header: format!("Content-Length: {}", body.len()),
        body,
    }
}

fn isolate_request<T, F>(mut stream: TcpStream, task: T, fs_prep: F) -> isize
where
    T: FnOnce(&TcpStream) -> PotatoResponse,
    F: FnOnce(), // TODO return type?
{
    net_prep();
    fs_prep();
    let response = task(&stream).to_http_response();
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
    0
}

fn fs_prep() {
    let uid = unistd::getuid();
    let pid = unistd::getpid();
    println!("fs prep pid: {}", pid);
    let rootfs = format!("/var/run/user/{}/potato/{}", uid, pid);
    println!("rootfs: {}", rootfs);

    // TODO handle error
    fs::create_dir_all(rootfs.as_str()).unwrap();
    unistd::chroot(rootfs.as_str()).unwrap();
    unistd::chdir(".").unwrap();
}

fn net_prep() {
    net::veth();
    net::bridge();

    let out = Command::new("bridge").arg("link").output().expect("wdf");
    io::stdout().write_all(&out.stdout).unwrap();
    io::stderr().write_all(&out.stderr).unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let get = b"GET / HTTP/1.1\r\n";
    let get_ns = b"GET /ns HTTP/1.1\r\n";

    const STACK_SIZE: usize = 1024 * 1024;
    let ref mut stack: [u8; STACK_SIZE] = [0; STACK_SIZE];

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    if buffer.starts_with(get) {
        let cb = || isolate_request(stream, get_hostname, fs_prep);
        let flags = libc::CLONE_NEWUTS | libc::CLONE_NEWNET;
        clone::clone_proc_newns(cb, stack, flags);
    } else if buffer.starts_with(get_ns) {
        let cb = || isolate_request(stream, set_and_get_hostname, fs_prep);
        let flags = libc::CLONE_NEWUTS
            | libc::CLONE_NEWNET
            | libc::CLONE_VM
            | libc::CLONE_THREAD
            | libc::CLONE_SIGHAND;
        clone::clone_proc_newns(cb, stack, flags);
    }
}
