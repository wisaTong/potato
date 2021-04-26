use lazy_static::lazy_static;
use libpotato::nix;
use nix::unistd;

use potato_ws::request::{HttpRequestMethod::*, PotatoRequest};
use potato_ws::response::PotatoResponse;
use potato_ws::server::PotatoServer;

lazy_static! {
    static ref RUNTIME_DIR: String = {
        let uid = unistd::getuid();
        format!("/var/run/user/{}/potato", uid)
    };
}

fn main() {
    let potato_server = PotatoServer::new("8000", &RUNTIME_DIR, true);
    potato_server.add_handler(GET, "/hello", hello).start()
}

fn hello(_: PotatoRequest) -> PotatoResponse {
    let res = PotatoResponse::new();
    let body = "Hello World!".as_bytes();
    res.set_status("200 OK")
        .add_body(body.to_owned())
        .add_header("Content-Length", &body.len().to_string())
}
