use lazy_static::lazy_static;
use libpotato::nix;
use nix::unistd;

use potato_ws::isolation::IsolationSetting;
use potato_ws::request::{HttpRequestMethod::*, PotatoRequest};
use potato_ws::response::PotatoResponse;
use potato_ws::server::PotatoServer;

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;

lazy_static! {
    static ref RUNTIME_DIR: String = {
        let uid = unistd::getuid();
        format!("/var/run/user/{}/potato", uid)
    };
    static ref STATIC_DIR: String = {
        let static_dir = env::var("STATIC_DIR").unwrap_or("STATIC_DIR not found".to_string());
        format!("{}", static_dir)
    };
}

fn main() {
    let isolation = IsolationSetting::new().add_bind_mount_point(&STATIC_DIR, "resources");

    let potato_server = PotatoServer::new("8000", &RUNTIME_DIR, true);
    potato_server
        .add_default_handler_with_isolation(serve_file, Some(isolation))
        .add_handler(GET, "/hello", hello)
        .add_handler(GET, "/hi", hi)
        .add_handler(POST, "/hanoi", hanoi)
        .add_handler(POST, "/add", simple_add)
        .add_handler(POST, "/sort", bubble_sort)
        .start()
}

fn hello(_: PotatoRequest) -> PotatoResponse {
    let res = PotatoResponse::new();
    let body = "Hello World!".as_bytes();
    res.set_status("200 OK")
        .add_body(body.to_owned())
        .add_header("Content-Length", &body.len().to_string())
}

fn hi(_: PotatoRequest) -> PotatoResponse {
    let res = PotatoResponse::new();
    let body = "Hi World".as_bytes();
    res.set_status("200 OK")
        .add_body(body.to_owned())
        .add_header("Content-Length", &body.len().to_string())
}

fn simple_add(req: PotatoRequest) -> PotatoResponse {
    let res = PotatoResponse::new();

    match req.body {
        Some(body) => {
            let num_str = body;
            let num_str = String::from_utf8_lossy(&num_str);
            let list = num_str.split(",");

            let mut result = 0;
            for num in list {
                result += num.parse::<i32>().unwrap();
            }

            let body = result.to_string().as_bytes().to_owned();
            res.set_status("200 OK")
                .add_header("Content-Length", &body.len().to_string())
                .add_body(body)
        }
        None => res.set_status("400 Bad Request"),
    }
}

fn compute_hanoi(num: i32, from: i32, to: i32, via: i32) {
    if num > 0 {
        compute_hanoi(num - 1, from, via, to);
        compute_hanoi(num - 1, via, to, from);
    }
}

fn hanoi(req: PotatoRequest) -> PotatoResponse {
    let res = PotatoResponse::new();

    match req.body {
        Some(body) => {
            let num_str = body;
            let num_str = String::from_utf8_lossy(&num_str);
            compute_hanoi(num_str.parse::<i32>().unwrap(), 1, 2, 3);

            let result = "Success";
            let body = result.to_string().as_bytes().to_owned();
            res.set_status("200 OK")
                .add_header("Content-Length", &body.len().to_string())
                .add_body(body)
        }
        None => res.set_status("400 Bad Request"),
    }
}

fn bubble_sort(req: PotatoRequest) -> PotatoResponse {
    let res = PotatoResponse::new();

    match req.body {
        Some(body) => {
            let num_str = body;
            let num_str = String::from_utf8_lossy(&num_str);
            let list = num_str.split(",");

            let mut vec = Vec::<i32>::new();
            for num in list {
                vec.push(num.parse::<i32>().unwrap());
            }

            for i in 0..vec.len() {
                for j in 0..vec.len() - 1 - i {
                    if vec[j] > vec[j + 1] {
                        vec.swap(j, j + 1);
                    }
                }
            }

            let mut result = String::new();
            for k in vec {
                result.push_str(&k.to_string());
                result = result + " ";
            }

            let body = result.to_string().as_bytes().to_owned();
            res.set_status("200 OK")
                .add_header("Content-Length", &body.len().to_string())
                .add_body(body)
        }
        None => res.set_status("400 Bad Request"),
    }
}

fn serve_file(req: PotatoRequest) -> PotatoResponse {
    let res = PotatoResponse::new();
    let filename = format!("/resources/{}", req.path);

    let mut file: File;
    if Path::new(&filename).exists() {
        file = File::open(filename).unwrap();
        let mut contents = String::new();
        // read the whole file
        match file.read_to_string(&mut contents) {
            Ok(_) => res
                .set_status("200")
                .add_body(contents.as_bytes().to_owned()),
            Err(_) => res.set_status("500 Internal Server Error"),
        }
    } else {
        res.set_status("404 Not Found")
    }
}
