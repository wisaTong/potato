use lazy_static::lazy_static;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::Read;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;

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

lazy_static! {
    static ref STATIC_DIR: String = {
        let static_dir = env::var("STATIC_DIR").unwrap_or("STATIC_DIR not found".to_string());
        format!("{}", static_dir)
    };
}

fn main() {
    let listener = TcpListener::bind("127.0.0.2:8002").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let get_add = b"GET /add HTTP/1.1\r\n";
    let get_hanoi = b"GET /hanoi HTTP/1.1\r\n";
    let get_bubble = b"GET /bubble HTTP/1.1\r\n";

    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    if buffer.starts_with(get_add) {
        // let response = 
        // stream.write(response.as_bytes()).unwrap();
        // stream.flush().unwrap();
    } else if buffer.starts_with(get_hanoi) {
        // let response = 
        // stream.write(response.as_bytes()).unwrap();
        // stream.flush().unwrap();
    } else if buffer.starts_with(get_bubble) {
        // let response = 
        // stream.write(response.as_bytes()).unwrap();
        // stream.flush().unwrap();
    } else {
        let s = str::from_utf8(&buffer).expect("can't covert utf8 to str");
        let (_, back) = s.split_at(s.find("/").unwrap());
        let (result, _) = back.split_at(s.find("H").unwrap() - 4);
        let response = serve_html_file(&stream, result.trim().to_string()).to_http_response();
        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}

fn check_file(file: String) -> Result<(), Box<std::error::Error>> {
    let suspend_file_name = format!("{}{}", STATIC_DIR.to_string(), file);
    let _suspend_file = File::open(suspend_file_name)?;

    Ok(())
}

fn serve_html_file(_: &TcpStream, file_name: String) -> PotatoResponse {
    let filename = format!("{}{}", STATIC_DIR.to_string(), file_name);
    let status_code;
    let body;
    let header = "content-type: text/html; charset=UTF-8".to_string();

    let checker = check_file(file_name);
    let mut file: File;
    if checker.is_ok() {
        file = File::open(filename).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read the file");
        body = contents.to_string();
        status_code = "200 Ok".to_string();
    } else {
        eprintln!("{} not found", filename);
        body = format!("505 Error");
        status_code = "505".to_string();
    }

    PotatoResponse {
        status_code,
        header,
        body,
    }
}
