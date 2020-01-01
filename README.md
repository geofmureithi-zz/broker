## Biller - Front-end Real-time Message Bus

[![crates.io](https://meritbadge.herokuapp.com/biller)](https://crates.io/crates/biller)

### Purpose

The purpose of this library is to provide a real-time front-end message bus using SSE and a JSON API. 

Biller is born from the need that rather than building a complex REST API with web-sockets to provide reactive web forms there is a simpler way.

Biller follows an insert-only/publish/subscribe paradigm rather than a REST CRUD paradigm. 

In Biller you insert an event and its data via a JSON POST request (/insert). Biller publishes the latest event to an event stream via SSE (/events) and keeps all older versions in its database that can be viewed in a JSON GET request (/audit/{event}).

When the client first subscribes to the SSE connection (/events) all the latest events and data is sent to the client. Combined with sending the latest event via SSE when subscribed negates any necessity to do any GET API requests in the lifecycle of events.

The side-effect of this system is that the latest event is the schema. Old events are saved in the database and are not changed but the latest event is the schema for the front-end. This is pure NoSQL as the backend is agnostic to the event data.

### Features

* Real-time Event Stream
* Stateful immutable event persistence
* JSON POST API to insert events 
* Sync latest events on client connection
* Audit log of event inserts

### Use

```rust
use biller::{biller_run};

#[actix_rt::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    biller_run("*".to_owned()).await
}
```

- the only param is the origin you want to allow - wildcard for all
- the PORT needs to passed in as an environment variable


### Example

- ``` make ```
- ``` make client ```

### Demo

- https://biller-demo.apibill.me
