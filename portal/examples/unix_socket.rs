#![deny(warnings)]

use tokio::net::UnixListener;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let mut listener = UnixListener::bind("/tmp/portal.sock").unwrap();
    let incoming = listener.incoming();
    portal::serve(portal::fs::dir("examples/dir"))
        .run_incoming(incoming)
        .await;
}
