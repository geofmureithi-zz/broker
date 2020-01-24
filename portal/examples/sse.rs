use futures::StreamExt;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::interval;
use portal::{sse::ServerSentEvent, Filter};

// create server-sent event
fn sse_counter(counter: u64) -> Result<impl ServerSentEvent, Infallible> {
    Ok(portal::sse::data(counter))
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let routes = portal::path("ticks").and(portal::get()).map(|| {
        let mut counter: u64 = 0;
        // create server event source
        let event_stream = interval(Duration::from_secs(1)).map(move |_| {
            counter += 1;
            sse_counter(counter)
        });
        // reply using server-sent events
        portal::sse::reply(event_stream)
    });

    portal::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
