#![deny(warnings)]

use serde_derive::{Deserialize, Serialize};

use portal::Filter;

#[derive(Deserialize, Serialize)]
struct Employee {
    name: String,
    rate: u32,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // POST /employees/:rate  {"name":"Sean","rate":2}
    let promote = portal::post()
        .and(portal::path("employees"))
        .and(portal::path::param::<u32>())
        // Only accept bodies smaller than 16kb...
        .and(portal::body::content_length_limit(1024 * 16))
        .and(portal::body::json())
        .map(|rate, mut employee: Employee| {
            employee.rate = rate;
            portal::reply::json(&employee)
        });

    portal::serve(promote).run(([127, 0, 0, 1], 3030)).await
}
