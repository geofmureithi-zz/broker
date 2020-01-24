#![deny(warnings)]
use portal::http::header::{HeaderMap, HeaderValue};
use portal::Filter;

#[tokio::test]
async fn header() {
    let header = portal::reply::with::header("foo", "bar");

    let no_header = portal::any().map(portal::reply).with(&header);

    let req = portal::test::request();
    let resp = req.reply(&no_header).await;
    assert_eq!(resp.headers()["foo"], "bar");

    let prev_header = portal::reply::with::header("foo", "sean");
    let yes_header = portal::any().map(portal::reply).with(prev_header).with(header);

    let req = portal::test::request();
    let resp = req.reply(&yes_header).await;
    assert_eq!(resp.headers()["foo"], "bar", "replaces header");
}

#[tokio::test]
async fn headers() {
    let mut headers = HeaderMap::new();
    headers.insert("server", HeaderValue::from_static("portal"));
    headers.insert("foo", HeaderValue::from_static("bar"));

    let headers = portal::reply::with::headers(headers);

    let no_header = portal::any().map(portal::reply).with(&headers);

    let req = portal::test::request();
    let resp = req.reply(&no_header).await;
    assert_eq!(resp.headers()["foo"], "bar");
    assert_eq!(resp.headers()["server"], "portal");

    let prev_header = portal::reply::with::header("foo", "sean");
    let yes_header = portal::any().map(portal::reply).with(prev_header).with(headers);

    let req = portal::test::request();
    let resp = req.reply(&yes_header).await;
    assert_eq!(resp.headers()["foo"], "bar", "replaces header");
}

#[tokio::test]
async fn default_header() {
    let def_header = portal::reply::with::default_header("foo", "bar");

    let no_header = portal::any().map(portal::reply).with(&def_header);

    let req = portal::test::request();
    let resp = req.reply(&no_header).await;

    assert_eq!(resp.headers()["foo"], "bar");

    let header = portal::reply::with::header("foo", "sean");
    let yes_header = portal::any().map(portal::reply).with(header).with(def_header);

    let req = portal::test::request();
    let resp = req.reply(&yes_header).await;

    assert_eq!(resp.headers()["foo"], "sean", "doesn't replace header");
}
