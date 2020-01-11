mod lib;
use lib::{broker_run};

#[actix_rt::main]
#[cfg_attr(tarpaulin, skip)]
async fn main() -> std::result::Result<(), std::io::Error> {
    broker_run().await
}
