#![deny(warnings)]

use serde_derive::Deserialize;
use std::collections::HashMap;
use portal::Filter;

#[tokio::test]
async fn query() {
    let as_map = portal::query::<HashMap<String, String>>();

    let req = portal::test::request().path("/?foo=bar&baz=quux");

    let extracted = req.filter(&as_map).await.unwrap();
    assert_eq!(extracted["foo"], "bar");
    assert_eq!(extracted["baz"], "quux");
}

#[tokio::test]
async fn query_struct() {
    let as_struct = portal::query::<MyArgs>();

    let req = portal::test::request().path("/?foo=bar&baz=quux");

    let extracted = req.filter(&as_struct).await.unwrap();
    assert_eq!(
        extracted,
        MyArgs {
            foo: Some("bar".into()),
            baz: Some("quux".into())
        }
    );
}

#[tokio::test]
async fn empty_query_struct() {
    let as_struct = portal::query::<MyArgs>();

    let req = portal::test::request().path("/?");

    let extracted = req.filter(&as_struct).await.unwrap();
    assert_eq!(
        extracted,
        MyArgs {
            foo: None,
            baz: None
        }
    );
}

#[tokio::test]
async fn missing_query_struct() {
    let as_struct = portal::query::<MyArgs>();

    let req = portal::test::request().path("/");

    let extracted = req.filter(&as_struct).await.unwrap();
    assert_eq!(
        extracted,
        MyArgs {
            foo: None,
            baz: None
        }
    );
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct MyArgs {
    foo: Option<String>,
    baz: Option<String>,
}

#[tokio::test]
async fn required_query_struct() {
    let as_struct = portal::query::<MyRequiredArgs>();

    let req = portal::test::request().path("/?foo=bar&baz=quux");

    let extracted = req.filter(&as_struct).await.unwrap();
    assert_eq!(
        extracted,
        MyRequiredArgs {
            foo: "bar".into(),
            baz: "quux".into()
        }
    );
}

#[tokio::test]
async fn missing_required_query_struct_partial() {
    let as_struct = portal::query::<MyRequiredArgs>();

    let req = portal::test::request().path("/?foo=something");

    let extracted = req.filter(&as_struct).await;
    assert!(extracted.is_err())
}

#[tokio::test]
async fn missing_required_query_struct_no_query() {
    let as_struct = portal::query::<MyRequiredArgs>().map(|_| portal::reply());

    let req = portal::test::request().path("/");

    let res = req.reply(&as_struct).await;
    assert_eq!(res.status(), 400);
    assert_eq!(res.body(), "Invalid query string");
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct MyRequiredArgs {
    foo: String,
    baz: String,
}

#[tokio::test]
async fn raw_query() {
    let as_raw = portal::query::raw();

    let req = portal::test::request().path("/?foo=bar&baz=quux");

    let extracted = req.filter(&as_raw).await.unwrap();
    assert_eq!(extracted, "foo=bar&baz=quux".to_owned());
}
