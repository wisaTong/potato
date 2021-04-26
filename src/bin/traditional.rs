use lazy_static::lazy_static;
use libpotato::nix;

use potato_ws::request::{HttpRequestMethod::*, PotatoRequest};
use potato_ws::response::PotatoResponse;
use potato_ws::server::PotatoServer;

use std::env;
use std::fs::File;
use std::io::Read;

lazy_static! {
    static ref RUNTIME_DIR: String = {
        let uid = nix::unistd::getuid();
        format!("/var/run/user/{}/potato", uid)
    };
    static ref STATIC_DIR: String = {
        let static_dir = env::var("STATIC_DIR").unwrap_or("STATIC_DIR not found".to_string());
        format!("{}", static_dir)
    };
}

fn main() {
    let potato_server = PotatoServer::new("8000", &RUNTIME_DIR);
    potato_server
        .add_default_handler(serve_file)
        .add_handler(GET, "/hello", hello)
        .add_handler(GET, "/hi", hi)
        .add_handler(POST, "/hanoi", hanoi)
        .start();
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

fn hanoi(req: PotatoRequest) -> PotatoResponse {
    let res = PotatoResponse::new();
    res.set_status("200 Ok").add_body(req.body.unwrap())
}

fn check_file(file: String) -> Result<(), Box<std::error::Error>> {
    let suspend_file_name = format!("{}{}", STATIC_DIR.to_string(), file);
    let _suspend_file = File::open(suspend_file_name)?;

    Ok(())
}

fn serve_file(req: PotatoRequest) -> PotatoResponse {
    let res = PotatoResponse::new();
    let filename = format!("{}{}", STATIC_DIR.to_string(), req.path);

    let checker = check_file(req.path);
    let mut file: File;
    if checker.is_ok() {
        file = File::open(filename).unwrap();
        let mut contents = String::new();
        // read the whole file
        file.read_to_string(&mut contents);
        res.set_status("200")
            .add_body(contents.as_bytes().to_owned())
    } else {
        eprintln!("{} not found", filename);
        res.set_status("500")
    }
}
