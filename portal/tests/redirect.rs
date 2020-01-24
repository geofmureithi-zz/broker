#![deny(warnings)]
use portal::{http::Uri, Filter};

#[tokio::test]
async fn redirect_uri() {
    let over_there = portal::any().map(|| portal::redirect(Uri::from_static("/over-there")));

    let req = portal::test::request();
    let resp = req.reply(&over_there).await;

    assert_eq!(resp.status(), 301);
    assert_eq!(resp.headers()["location"], "/over-there");
}
