use lazy_static::lazy_static;
use libpotato::nix;

use potato_ws::request::{HttpRequestMethod::*, PotatoRequest};
use potato_ws::response::PotatoResponse;
use potato_ws::server::PotatoServer;

lazy_static! {
    static ref RUNTIME_DIR: String = {
        let uid = nix::unistd::getuid();
        format!("/var/run/user/{}/potato", uid)
    };
}

fn main() {
    let potato_server = PotatoServer::new("8000", &RUNTIME_DIR);
    potato_server
        .add_handler(GET, "/hello", hello)
        .add_handler(GET, "/hi", hi)
        .start();
}

fn hello<'a>(_: PotatoRequest) -> PotatoResponse<'a> {
    let res = PotatoResponse::new();
    let body = "Hello World!".as_bytes();
    res.set_status("200 OK")
        .add_body(body)
        .add_header("Content-Length", &body.len().to_string())
}

fn hi<'a>(_: PotatoRequest) -> PotatoResponse<'a> {
    let res = PotatoResponse::new();
    let body = "Hi World".as_bytes();
    res.set_status("200 OK")
        .add_body(body)
        .add_header("Content-Length", &body.len().to_string())
}
