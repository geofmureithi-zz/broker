mod lib;
use lib::{sse_start};

#[actix_rt::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    sse_start("*".to_owned()).await
}
