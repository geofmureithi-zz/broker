mod lib;
use lib::{broker_run};

#[actix_rt::main]
async fn main() -> () {
    let _ = broker_run("*".to_owned()).await;
}
