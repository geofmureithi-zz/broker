mod lib;
use lib::{broker_run, Config};

#[actix_rt::main]
async fn main() -> () {
    // get origin env var
    let config = envy::from_env::<Config>().unwrap();
    let origin = config.origin;
      
    let _ = broker_run(origin).await;
}
