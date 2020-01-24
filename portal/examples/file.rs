#![deny(warnings)]

use portal::Filter;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let readme = portal::get()
        .and(portal::path::end())
        .and(portal::fs::file("./README.md"));

    // dir already requires GET...
    let examples = portal::path("ex").and(portal::fs::dir("./examples/"));

    // GET / => README.md
    // GET /ex/... => ./examples/..
    let routes = readme.or(examples);

    portal::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
