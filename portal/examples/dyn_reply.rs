#![deny(warnings)]
use portal::{http::StatusCode, Filter};

async fn dyn_reply(word: String) -> Result<Box<dyn portal::Reply>, portal::Rejection> {
    if &word == "hello" {
        // a cast is needed for now, see https://github.com/rust-lang/rust/issues/60424
        Ok(Box::new("world") as Box<dyn portal::Reply>)
    } else {
        Ok(Box::new(StatusCode::BAD_REQUEST) as Box<dyn portal::Reply>)
    }
}

#[tokio::main]
async fn main() {
    let routes = portal::path::param().and_then(dyn_reply);

    portal::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
