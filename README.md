# Biller - Front-end Message Bus using SSE

[![crates.io](https://meritbadge.herokuapp.com/biller](https://crates.io/crates/biller)

## Use

```rust
use biller::{biller_run};

#[actix_rt::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    biller_run("*".to_owned()).await
}
```

- the only param is the origin you want to allow - wildcard for all
- the PORT needs to passed in as an environment variable


## Example

- run ``` make ``` and ``` make client ``` in two different terminal windows to run example

## Example

- https://biller-demo.apibill.me
