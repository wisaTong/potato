use lazy_static::lazy_static;
use libc;
use nix::sys::{signal, wait};
use nix::unistd;
use potato::{clone, idmap, net::{self, set_inside_network}};
use std::{fs, string};
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

const IP_BRIDGE: &'static str = "10.0.0.0/24";

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

extern "C" fn handl_sigchld(_: libc::c_int) {
    wait::wait().unwrap();
}

lazy_static! {
    static ref RUNTIME_DIR: String = {
        let uid = unistd::getuid();
        format!("/var/run/user/{}/potato", uid)
    };
}

fn main() {
    // install signal handler
    let handler = signal::SigHandler::Handler(handl_sigchld);
    unsafe { signal::signal(signal::SIGCHLD, handler) }.unwrap();

    // create potato dir for each request if not exist
    fs::create_dir_all(RUNTIME_DIR.as_str()).expect("Faild to create runtime dir");

    //create bridge for network isolate
    net::prep_bridge(IP_BRIDGE.to_string());

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

fn isolate_request<T, F, N>(mut stream: TcpStream, task: T, fs_prep: F, net_prep: N)
where
    T: FnOnce(&TcpStream) -> PotatoResponse,
    F: FnOnce(unistd::Pid) -> String,
    N: FnOnce(String, u32), // TODO paramenter, return type?
{
    const STACK_SIZE: usize = 1024 * 1024;
    let ref mut stack: [u8; STACK_SIZE] = [0; STACK_SIZE];

    let cb = || {
        let response = task(&stream).to_http_response();
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();

        // clean up
        0
    };

    let flags = libc::CLONE_NEWUSER
        | libc::CLONE_NEWUTS
        | libc::CLONE_NEWNET
        | libc::CLONE_NEWNS
        | libc::CLONE_NEWPID
        | libc::CLONE_NEWIPC
        | libc::CLONE_NEWCGROUP
        | libc::SIGCHLD
        | libc::SIGSTOP;

    match clone::clone_proc_newns(cb, stack, flags) {
        Ok(pid) => {
            // // FIXME EVERYTHING HERE BREAKS
            // // id mapping
            // idmap::UidMapper::new()
            //     .add(0, 1000, 1)
            //     .write_newuidmap(1)
            //     .unwrap();

            // // FIXME should not be using child pid
            // // waitting for Book's implementation of directory numbering
            // let rootfs = fs_prep(unistd::Pid::from_raw(pid));

            // // FIXME try to not unwrap?
            // unistd::chroot(rootfs.as_str()).unwrap();
            // unistd::chdir(".").unwrap();

            // // network setup
            // net_prep(ip: String, pid: u32);

            // // TODO send sigcont
            // // network setup inside clone
            // net::set_inside_network(ip[1].to_string());
        }
        Err(e) => {
            handle_req_error(stream, e.to_string().as_str());
        }
    }
}

fn fs_prep(pid: unistd::Pid) -> String {
    let rootfs = format!("{}/{}", *RUNTIME_DIR, pid.as_raw());
    fs::create_dir_all(rootfs.as_str()).unwrap(); // TODO handle error
    rootfs
}

fn net_prep(veth: String, pid: u32) {
    net::prep_network_stack(veth, pid);
    // // TODO set up veth inside clone
    // net::set_inside_network(ip);
}

fn handle_req_error(mut stream: TcpStream, message: &str) {
    let header = format!("Content-Length: {}", message.len());
    let response = format!(
        "HTTP/1.1 500 Internal Server Error\r\n{}\r\n\r\n{}",
        header, message
    );
    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let get = b"GET / HTTP/1.1\r\n";
    let get_ns = b"GET /ns HTTP/1.1\r\n";

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    if buffer.starts_with(get) {
        isolate_request(stream, get_hostname, fs_prep, net_prep);
    } else if buffer.starts_with(get_ns) {
        isolate_request(stream, set_and_get_hostname, fs_prep, net_prep);
    }
}
