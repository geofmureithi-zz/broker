mod lib;
use lib::{biller_run};

#[actix_rt::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    biller_run("*".to_owned()).await
}
