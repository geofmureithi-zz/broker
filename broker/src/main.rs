mod lib;
use lib::broker;

#[tokio::main]
pub async fn main() {
    broker().await
}