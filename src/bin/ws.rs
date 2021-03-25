use nix::sched;
use nix::unistd;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        route(stream);
    }
}

fn route(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let ns = b"GET /ns HTTP/1.1\r\n";

    if buffer.starts_with(get) {
        thread::spawn(|| hello_hostname(stream));
    } else if buffer.starts_with(ns) {
        hello_ns_hostname(stream);
    } else {
        let status_line = "HTTP/1.1 404 Not Found\r\n\r\n";
        let response = format!("{}", status_line);
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    };
}

enum Error {
    GetHostname,
    SetHostname,
    Unshare,
}

fn hello_hostname(mut stream: TcpStream) {
    match get_hostname() {
        Ok(hostname) => {
            let status_line = "HTTP/1.1 200 OK\r\n\r\n";
            let response = format!("{}{}", status_line, hostname);
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        Err(_) => {
            let status_line = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
            let response = format!("{}", status_line);
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}

fn hello_ns_hostname(mut stream: TcpStream) {
    if let Err(_) = unshare_uts() {
        let status_line = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
        let response = format!("{}{}", status_line, "Error unsharing uts");
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return;
    }

    if let Err(_) = set_hostname("inside new uts") {
        let status_line = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
        let response = format!("{}{}", status_line, "Error setting hostname");
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return;
    }

    match get_hostname() {
        Ok(hostname) => {
            let status_line = "HTTP/1.1 200 OK\r\n\r\n";
            let response = format!("{}{}", status_line, hostname);
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
        Err(_) => {
            let status_line = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
            let response = format!("{}", status_line);
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}

fn get_hostname() -> Result<String, Error> {
    let mut buf = [0u8; 64];
    let hostname_cstr = match unistd::gethostname(&mut buf) {
        Ok(cstr) => cstr,
        Err(_) => return Err(Error::GetHostname),
    };
    let hostname = match hostname_cstr.to_str() {
        Ok(res) => res,
        Err(_) => return Err(Error::GetHostname),
    };

    Ok(hostname.to_string())
}

fn set_hostname(hostname: &str) -> Result<(), Error> {
    match unistd::sethostname(hostname) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::SetHostname),
    }
}

fn unshare_uts() -> Result<(), Error> {
    match sched::unshare(sched::CloneFlags::CLONE_NEWUTS) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::Unshare),
    }
}
