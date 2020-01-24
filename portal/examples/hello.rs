#![deny(warnings)]
use portal::Filter;

#[tokio::main]
async fn main() {
    // Match any request and return hello world!
    let routes = portal::any().map(|| "Hello, World!");

    portal::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
