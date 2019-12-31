# SSE server using Actix Web

[![crates.io](https://meritbadge.herokuapp.com/sse-server)](https://crates.io/crates/sse-server)

## Use

```rust
use sse_server::{sse_start};

#[actix_rt::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    sse_start("*".to_owned()).await
}
```

- the only param is the origin you want to allow - wildcard for all
- the PORT needs to passed in as an environment variable


## Example

- run ``` make ``` and ``` cd example && npm i && npm start ``` to run example

## Example

- https://sse-demo.mynextlevel.dev
